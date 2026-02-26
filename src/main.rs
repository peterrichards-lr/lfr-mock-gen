mod cli;
mod core;
mod utils;

use crate::cli::{App, AppCommands};
use crate::core::{LiferayWorkspace, Workspace};
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize, Debug)]
struct Article {
    title: String,
    description: String,
    body: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = App::parse();

    let ws = LiferayWorkspace {
        current_dir: std::env::current_dir().unwrap_or_default(),
    };

    match args.command {
        AppCommands::Env { target } => {
            let root = ws.find_root().map_err(anyhow::Error::msg)?;
            println!("Environment check for {:?} in root: {:?}", target, root);
            let _ = utils::find_elements_by_name;
        }
        AppCommands::Data { 
            force, 
            api_env, 
            group_id, 
            liferay_url, 
            liferay_user, 
            liferay_pass 
        } => {
            println!("Data operation initiated (Force={})", force);
            
            generate_mock_data(
                &api_env, 
                group_id, 
                &liferay_url, 
                &liferay_user, 
                &liferay_pass
            ).await?;
        }
    }

    Ok(())
}

/// Helper function to negotiate the Content Structure by its Name
async fn get_structure_or_fail(
    client: &Client,
    liferay_url: &str,
    group_id: u64,
    liferay_user: &str,
    liferay_pass: &str,
) -> anyhow::Result<u64> {
    let target_name = "AI Generated Article";
    
    // Fetch all structures for the site
    let get_url = format!(
        "{}/o/headless-delivery/v1.0/sites/{}/content-structures",
        liferay_url.trim_end_matches('/'),
        group_id
    );

    let res = client.get(&get_url).basic_auth(liferay_user, Some(liferay_pass)).send().await?;

    if res.status().is_success() {
        let text = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        // Iterate through the returned items and look for our target name
        if let Some(items) = json["items"].as_array() {
            for item in items {
                if item["name"].as_str() == Some(target_name) {
                    if let Some(id) = item["id"].as_u64() {
                        println!("✅ Found Content Structure '{}' (ID: {})", target_name, id);
                        return Ok(id);
                    }
                }
            }
        }
    }

    // If not found, gracefully fail and provide a SIMPLIFIED UI setup guide
    anyhow::bail!(
        "\n❌ Content Structure not found, and Liferay does not allow creating them via API.\n\n\
        HOW TO FIX THIS (One-Time Setup):\n\
        1. Log into your Liferay UI and go to your Site.\n\
        2. Navigate to Content & Data -> Web Content -> Structures.\n\
        3. Click the '+' button to create a new Structure.\n\
        4. Name it EXACTLY: {}\n\
        5. Drag a 'Rich Text' field into the builder.\n\
        6. Edit the field's settings, go to the 'Advanced' tab, and name the Field Reference 'content'.\n\
        7. Save the structure and re-run this tool!",
        target_name
    )
}

/// Handles fetching data from Gemini and pushing it to Liferay
async fn generate_mock_data(
    api_env: &str,
    group_id: u64,
    liferay_url: &str,
    liferay_user: &str,
    liferay_pass: &str,
) -> anyhow::Result<()> {
    
    let api_key = env::var(api_env).map_err(|_| {
        anyhow::anyhow!("Environment variable '{}' is not set. Please export it.", api_env)
    })?;

    let client = Client::new();

    // Ensure the Structure exists BEFORE we call Gemini
    let structure_id = get_structure_or_fail(&client, liferay_url, group_id, liferay_user, liferay_pass).await?;

    println!("Fetching generated data from Gemini...");
    let gemini_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = "Generate 20 creative, realistic news articles for a blog. Return EXACTLY a JSON array of objects with keys: 'title', 'description', 'body'. Do not include markdown formatting.";
    
    let gemini_payload = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let gemini_res = client.post(&gemini_url).json(&gemini_payload).send().await?;
    let gemini_json: serde_json::Value = gemini_res.json().await?;

    let text_response = gemini_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("[]")
        .replace("```json", "")
        .replace("```", "");

    let articles: Vec<Article> = serde_json::from_str(text_response.trim())
        .map_err(|e| anyhow::anyhow!("Failed to parse Gemini JSON: {}", e))?;
    
    println!("Successfully generated {} articles. Pushing to Liferay...", articles.len());

    let liferay_endpoint = format!(
        "{}/o/headless-delivery/v1.0/sites/{}/structured-contents",
        liferay_url.trim_end_matches('/'),
        group_id
    );

    for (i, article) in articles.iter().enumerate() {
        let image_url = format!("https://picsum.photos/seed/headline{}{}/800/400", group_id, i);
        let content_body = format!(
            "<img src='{}' alt='Featured Image' style='max-width: 100%; height: auto; margin-bottom: 15px;' /><p>{}</p>",
            image_url, article.body
        );

        let liferay_payload = json!({
            "contentStructureId": structure_id, 
            "title": article.title,
            "description": article.description,
            "contentFields": [
                {
                    "name": "content",
                    "contentFieldValue": {
                        "data": content_body
                    }
                }
            ],
            "keywords": ["headline"] 
        });

        let response = client
            .post(&liferay_endpoint)
            .basic_auth(liferay_user, Some(liferay_pass))
            .json(&liferay_payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Created: {}", article.title);
        } else {
            let error_text = response.text().await?;
            eprintln!("❌ Failed to create '{}': {}", article.title, error_text);
        }
    }

    println!("\nFinished importing articles into Liferay!");
    Ok(())
}