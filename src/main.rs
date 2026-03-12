mod cli;
mod core;
mod utils;

use crate::cli::{App, AppCommands};
use crate::core::{LiferayProject, Workspace};
use crate::utils::xml;
use clap::Parser;
use reqwest::Client;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = App::parse();

    let ws = LiferayProject {
        current_dir: std::env::current_dir().unwrap_or_default(),
    };

    match args.command {
        AppCommands::Env { target } => {
            let root = ws.find_root()?;
            println!("Environment check for {:?} in root: {:?}", target, root);
            let _ = xml::find_elements_by_name;
        }
        AppCommands::Data {
            force,
            api_env,
            group_id,
            liferay_url,
            liferay_user,
            liferay_pass,
            structure_id,
            tone,
            purpose,
        } => {
            println!(
                "Data operation initiated (Force={}, Structure ID/Name={}, Tone={}, Purpose={})",
                force, structure_id, tone, purpose
            );

            generate_mock_data(
                &api_env,
                group_id,
                &liferay_url,
                &liferay_user,
                &liferay_pass,
                &structure_id,
                &tone,
                &purpose,
            )
            .await?;
        }
    }

    Ok(())
}

/// Helper function to find the Content Structure by its ID or Name and return its full definition
async fn get_structure_definition(
    client: &Client,
    liferay_url: &str,
    group_id: u64,
    liferay_user: &str,
    liferay_pass: &str,
    identifier: &str,
) -> anyhow::Result<serde_json::Value> {
    // Fetch all structures for the site
    let get_url = format!(
        "{}/o/headless-delivery/v1.0/sites/{}/content-structures",
        liferay_url.trim_end_matches('/'),
        group_id
    );

    let res = client
        .get(&get_url)
        .basic_auth(liferay_user, Some(liferay_pass))
        .send()
        .await?;

    if res.status().is_success() {
        let text = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)?;

        if let Some(items) = json["items"].as_array() {
            for item in items {
                let id = item["id"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let name = item["name"].as_str().unwrap_or_default();

                if id == identifier || name == identifier {
                    println!("✅ Found Content Structure '{}' (ID: {})", name, id);
                    return Ok(item.clone());
                }
            }
        }
    }

    anyhow::bail!(
        "❌ Content Structure '{}' not found in Site ID {}.",
        identifier,
        group_id
    )
}

/// Normalizes a Liferay Content Structure into a simple JSON Schema for Gemini
fn normalize_structure_to_schema(structure: &serde_json::Value) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = vec!["title".to_string()];

    // Title and description are standard for Structured Content
    properties.insert(
        "title".to_string(),
        json!({
            "type": "string",
            "description": "The title of the web content article"
        }),
    );
    properties.insert(
        "description".to_string(),
        json!({
            "type": "string",
            "description": "A short summary or description of the article"
        }),
    );

    if let Some(fields) = structure["contentStructureFields"].as_array() {
        for field in fields {
            if let Some(name) = field["name"].as_str() {
                let label = field["label"].as_str().unwrap_or(name);
                let data_type = field["dataType"].as_str().unwrap_or("string");

                let mut field_schema = match data_type {
                    "integer" | "number" | "double" => json!({
                        "type": "number",
                        "description": label
                    }),
                    "boolean" => json!({
                        "type": "boolean",
                        "description": label
                    }),
                    "date" => json!({
                        "type": "string",
                        "description": format!("{} (ISO 8601 date, e.g., YYYY-MM-DD)", label)
                    }),
                    "image" => json!({
                        "type": "string",
                        "description": format!("{} (A high-quality image URL from picsum.photos)", label)
                    }),
                    _ => {
                        // Check for Select/Radio options
                        if let Some(options) = field["nestedContentStructureFields"].as_array() {
                            let mut enum_values = Vec::new();
                            for opt in options {
                                if let Some(val) = opt["label"].as_str() {
                                    enum_values.push(val.to_string());
                                }
                            }

                            if !enum_values.is_empty() {
                                json!({
                                    "type": "string",
                                    "enum": enum_values,
                                    "description": label
                                })
                            } else {
                                json!({
                                    "type": "string",
                                    "description": label
                                })
                            }
                        } else {
                            json!({
                                "type": "string",
                                "description": label
                            })
                        }
                    }
                };

                // Handle Multiple Selection (if dataType is string but it's a 'checkbox' or similar)
                // In Liferay, this is often indicated by the 'localizable' or other flags,
                // but let's look for common patterns.
                if let Some(field_type) = field["type"].as_str() {
                    if field_type == "multiselect" || field_type == "checkbox_multiple" {
                        if let Some(obj) = field_schema.as_object_mut() {
                            let enum_vals = obj.remove("enum");
                            obj.insert("type".to_string(), json!("array"));
                            obj.insert(
                                "items".to_string(),
                                json!({
                                    "type": "string",
                                    "enum": enum_vals
                                }),
                            );
                        }
                    } else if field_type == "color" {
                        if let Some(obj) = field_schema.as_object_mut() {
                            obj.insert(
                                "description".to_string(),
                                json!(format!("{} (Hex color code, e.g., #FFFFFF)", label)),
                            );
                        }
                    }
                }

                properties.insert(name.to_string(), field_schema);
                required.push(name.to_string());
            }
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Handles fetching data from Gemini and pushing it to Liferay
async fn generate_mock_data(
    api_env: &str,
    group_id: u64,
    liferay_url: &str,
    liferay_user: &str,
    liferay_pass: &str,
    structure_identifier: &str,
    tone: &str,
    purpose: &str,
) -> anyhow::Result<()> {
    let api_key = env::var(api_env).map_err(|_| {
        anyhow::anyhow!(
            "Environment variable '{}' is not set. Please export it.",
            api_env
        )
    })?;

    let client = Client::new();

    // 1. Get the structure definition
    let structure = get_structure_definition(
        &client,
        liferay_url,
        group_id,
        liferay_user,
        liferay_pass,
        structure_identifier,
    )
    .await?;

    let structure_id = structure["id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Invalid structure ID"))?;

    // 2. Normalize to a JSON Schema
    let schema = normalize_structure_to_schema(&structure);
    let schema_str = serde_json::to_string_pretty(&schema)?;

    println!("Fetching generated data from Gemini based on structure schema...");
    let gemini_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "Generate 5 realistic and diverse mock {} entries with a {} tone. \
        Return EXACTLY a JSON array of objects that strictly follow this JSON schema: \n{}\n\
        GUIDELINES:\n\
        - Use creative and varied titles and descriptions suitable for a {}.\n\
        - For 'string' fields that look like content (e.g., 'body', 'content'), include realistic HTML with <p>, <h2>, <ul> tags.\n\
        - For 'image' fields, provide a unique URL from https://picsum.photos/ (e.g., https://picsum.photos/seed/abc/800/400).\n\
        - For 'date' fields, use ISO 8601 format (YYYY-MM-DD).\n\
        - Ensure numerical values are within realistic ranges.\n\
        - DO NOT include markdown formatting, backticks, or any text other than the JSON array.",
        purpose, tone, schema_str, purpose
    );

    let gemini_payload = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let gemini_res = client
        .post(&gemini_url)
        .json(&gemini_payload)
        .send()
        .await?;
    let gemini_json: serde_json::Value = gemini_res.json().await?;

    let text_response = gemini_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No text response from Gemini: {:?}", gemini_json))?
        .trim()
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();

    let articles: Vec<serde_json::Value> = serde_json::from_str(text_response).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse Gemini JSON: {}\nResponse: {}",
            e,
            text_response
        )
    })?;

    // 3. Validate response against schema
    let compiled_schema = jsonschema::JSONSchema::compile(&schema)
        .map_err(|e| anyhow::anyhow!("Invalid schema generated: {}", e))?;

    println!(
        "Successfully generated {} articles. Validating and pushing to Liferay...",
        articles.len()
    );

    let liferay_endpoint = format!(
        "{}/o/headless-delivery/v1.0/sites/{}/structured-contents",
        liferay_url.trim_end_matches('/'),
        group_id
    );

    for article in articles {
        // Validate against schema
        if let Err(errors) = compiled_schema.validate(&article) {
            let err_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            eprintln!(
                "⚠️ Skipping article due to schema validation errors: {:?}",
                err_msgs
            );
            continue;
        }

        // 4. Map the AI response back to Liferay's format
        let title = article["title"]
            .as_str()
            .unwrap_or("Untitled Article")
            .to_string();
        let description = article["description"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let mut content_fields = Vec::new();

        // Map fields from the article object (excluding title/description)
        if let Some(obj) = article.as_object() {
            for (key, value) in obj {
                if key == "title" || key == "description" {
                    continue;
                }

                // Determine field type from schema to handle specific mapping (like images)
                let field_schema = schema["properties"].get(key);
                let description = field_schema
                    .and_then(|s| s["description"].as_str())
                    .unwrap_or_default();

                let payload_value = if description.contains("picsum.photos") {
                    // It's an image field - Liferay Headless Delivery often supports simple URL strings
                    // or objects. We'll stick to a simple mapping for now but could expand to
                    // {"data": value, "alt": "..."} if needed.
                    json!({ "data": value })
                } else {
                    json!({ "data": value })
                };

                content_fields.push(json!({
                    "name": key,
                    "contentFieldValue": payload_value
                }));
            }
        }

        let liferay_payload = json!({
            "contentStructureId": structure_id,
            "title": title,
            "description": description,
            "contentFields": content_fields,
            "keywords": ["ai-generated"]
        });

        let response = client
            .post(&liferay_endpoint)
            .basic_auth(liferay_user, Some(liferay_pass))
            .json(&liferay_payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Created: {}", title);
        } else {
            let error_text = response.text().await?;
            eprintln!("❌ Failed to create '{}': {}", title, error_text);
        }
    }

    println!("\nFinished processing articles!");
    Ok(())
}
