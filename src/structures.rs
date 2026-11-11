//! Content Structure lookup.
//!
//! The lookup logic is written against the [`StructureApi`] trait so that it
//! can be tested without a Liferay instance; [`HttpStructureApi`] is the real
//! implementation backed by the Headless Delivery and Headless Admin User APIs.

use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::future::Future;

/// Page size used when listing a site's structures (Liferay defaults to 20).
pub const PAGE_SIZE: u64 = 100;

/// Upper bound on pages read per site, as a guard against a server that never
/// reports a last page.
const MAX_PAGES: u64 = 1000;

/// The Liferay calls needed to resolve a Content Structure.
pub(crate) trait StructureApi {
    /// `GET /o/headless-delivery/v1.0/content-structures/{id}`.
    /// Returns `Ok(None)` when the structure does not exist (HTTP 404).
    async fn get_structure(&self, id: u64) -> anyhow::Result<Option<Value>>;

    /// One page of `GET /o/headless-delivery/v1.0/sites/{siteId}/content-structures`.
    async fn get_site_structures_page(
        &self,
        site_id: u64,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Value>;

    /// Resolves the Global site's ID.
    async fn get_global_site_id(&self) -> anyhow::Result<u64>;
}

/// Returns the item whose `name` is exactly `name`, if any.
pub fn find_by_name<'a>(items: &'a [Value], name: &str) -> Option<&'a Value> {
    items
        .iter()
        .find(|item| item["name"].as_str() == Some(name))
}

/// Whether `page` (1-based) is the last page of a Liferay page response,
/// based on `lastPage`, falling back to `totalCount` and then to the number of
/// items returned.
pub fn is_last_page(response: &Value, page: u64, page_size: u64) -> bool {
    if let Some(last_page) = response["lastPage"].as_u64() {
        return page >= last_page;
    }
    if let Some(total) = response["totalCount"].as_u64() {
        return page.saturating_mul(page_size) >= total;
    }
    let count = response["items"].as_array().map_or(0, |items| items.len()) as u64;
    count < page_size
}

/// Reads pages 1, 2, ... via `fetch` until an item named `name` is found or
/// the last page has been read.
pub async fn find_by_name_paged<F, Fut>(
    mut fetch: F,
    name: &str,
    page_size: u64,
) -> anyhow::Result<Option<Value>>
where
    F: FnMut(u64) -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    for page in 1..=MAX_PAGES {
        let response = fetch(page).await?;
        let items = response["items"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if let Some(found) = find_by_name(items, name) {
            return Ok(Some(found.clone()));
        }
        if items.is_empty() || is_last_page(&response, page, page_size) {
            break;
        }
    }
    Ok(None)
}

/// Finds a Content Structure by numeric ID or exact name.
///
/// - A numeric identifier is fetched directly (this works for structures in
///   any site). If no structure has that ID, it is looked up as a name.
/// - A name is searched in every page of the site's structures, then in every
///   page of the Global site's structures. `global_site_id` overrides the
///   Global site ID that is otherwise resolved through the API.
pub(crate) async fn find_structure<A: StructureApi>(
    api: &A,
    group_id: u64,
    global_site_id: Option<u64>,
    identifier: &str,
) -> anyhow::Result<Value> {
    if let Ok(id) = identifier.parse::<u64>() {
        if let Some(structure) = api.get_structure(id).await? {
            report_found(&structure);
            return Ok(structure);
        }
    }

    let search_site = |site_id: u64| {
        find_by_name_paged(
            move |page| api.get_site_structures_page(site_id, page, PAGE_SIZE),
            identifier,
            PAGE_SIZE,
        )
    };

    if let Some(structure) = search_site(group_id).await? {
        report_found(&structure);
        return Ok(structure);
    }

    let global_id = match global_site_id {
        Some(id) => id,
        None => match api.get_global_site_id().await {
            Ok(id) => id,
            Err(e) => anyhow::bail!(
                "❌ Content Structure '{}' not found in Site ID {}, and the Global site could not be searched ({}). Pass --global-site-id to search it.",
                identifier,
                group_id,
                e
            ),
        },
    };

    if global_id != group_id {
        if let Some(structure) = search_site(global_id).await? {
            report_found(&structure);
            return Ok(structure);
        }
    }

    anyhow::bail!(
        "❌ Content Structure '{}' not found in Site ID {} or the Global site (ID {}).",
        identifier,
        group_id,
        global_id
    )
}

fn report_found(structure: &Value) {
    println!(
        "✅ Found Content Structure '{}' (ID: {})",
        structure["name"].as_str().unwrap_or_default(),
        structure["id"]
    );
}

/// [`StructureApi`] backed by Liferay's Headless APIs over HTTP Basic auth.
pub(crate) struct HttpStructureApi<'a> {
    pub client: &'a Client,
    pub base_url: &'a str,
    pub user: &'a str,
    pub pass: &'a str,
}

impl HttpStructureApi<'_> {
    async fn get(&self, path: &str) -> anyhow::Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        Ok(self
            .client
            .get(&url)
            .basic_auth(self.user, Some(self.pass))
            .send()
            .await?)
    }
}

async fn json_or_error(res: reqwest::Response, what: &str) -> anyhow::Result<Value> {
    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        anyhow::bail!("{} failed with HTTP {}: {}", what, status, text);
    }
    Ok(serde_json::from_str(&text)?)
}

impl StructureApi for HttpStructureApi<'_> {
    async fn get_structure(&self, id: u64) -> anyhow::Result<Option<Value>> {
        let res = self
            .get(&format!(
                "/o/headless-delivery/v1.0/content-structures/{id}"
            ))
            .await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        json_or_error(res, &format!("Fetching Content Structure {id}"))
            .await
            .map(Some)
    }

    async fn get_site_structures_page(
        &self,
        site_id: u64,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Value> {
        let res = self
            .get(&format!(
                "/o/headless-delivery/v1.0/sites/{site_id}/content-structures?page={page}&pageSize={page_size}"
            ))
            .await?;
        json_or_error(
            res,
            &format!("Listing Content Structures of Site ID {site_id}"),
        )
        .await
    }

    async fn get_global_site_id(&self) -> anyhow::Result<u64> {
        let res = self
            .get("/o/headless-admin-user/v1.0/sites/by-friendly-url-path/global")
            .await?;
        let site = json_or_error(res, "Resolving the Global site").await?;
        site["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Global site response has no numeric 'id'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::RefCell;
    use std::collections::HashMap;

    const SITE: u64 = 20117;
    const GLOBAL: u64 = 20097;

    fn structure(id: u64, name: &str) -> Value {
        json!({ "id": id, "name": name, "contentStructureFields": [] })
    }

    /// Builds Liferay-style page responses (`items`, `lastPage`, `page`,
    /// `pageSize`, `totalCount`) from a flat list of structures.
    fn pages(structures: Vec<Value>, page_size: u64) -> Vec<Value> {
        let total = structures.len() as u64;
        let last_page = total.div_ceil(page_size).max(1);
        let mut chunks: Vec<Vec<Value>> = structures
            .chunks(page_size as usize)
            .map(<[Value]>::to_vec)
            .collect();
        if chunks.is_empty() {
            chunks.push(Vec::new());
        }
        chunks
            .into_iter()
            .enumerate()
            .map(|(i, items)| {
                json!({
                    "items": items,
                    "lastPage": last_page,
                    "page": i as u64 + 1,
                    "pageSize": page_size,
                    "totalCount": total
                })
            })
            .collect()
    }

    fn many(start: u64, count: u64, prefix: &str) -> Vec<Value> {
        (0..count)
            .map(|i| structure(start + i, &format!("{prefix} {i}")))
            .collect()
    }

    #[derive(Default)]
    struct FakeApi {
        by_id: HashMap<u64, Value>,
        site_pages: HashMap<u64, Vec<Value>>,
        global_id: Option<u64>,
        page_requests: RefCell<Vec<(u64, u64)>>,
    }

    impl StructureApi for FakeApi {
        async fn get_structure(&self, id: u64) -> anyhow::Result<Option<Value>> {
            Ok(self.by_id.get(&id).cloned())
        }

        async fn get_site_structures_page(
            &self,
            site_id: u64,
            page: u64,
            page_size: u64,
        ) -> anyhow::Result<Value> {
            assert_eq!(page_size, PAGE_SIZE);
            self.page_requests.borrow_mut().push((site_id, page));
            let pages = self.site_pages.get(&site_id).cloned().unwrap_or_default();
            Ok(pages
                .get(page as usize - 1)
                .cloned()
                .unwrap_or_else(|| json!({ "items": [] })))
        }

        async fn get_global_site_id(&self) -> anyhow::Result<u64> {
            self.global_id
                .ok_or_else(|| anyhow::anyhow!("No site exists with friendly URL global"))
        }
    }

    #[test]
    fn find_by_name_matches_exact_name_only() {
        let items = vec![
            structure(1, "Basic Web Content Extra"),
            structure(2, "Basic Web Content"),
        ];
        assert_eq!(find_by_name(&items, "Basic Web Content").unwrap()["id"], 2);
        assert!(find_by_name(&items, "basic web content").is_none());
    }

    #[test]
    fn is_last_page_uses_last_page_then_total_count_then_item_count() {
        assert!(!is_last_page(&json!({ "lastPage": 3 }), 2, 100));
        assert!(is_last_page(&json!({ "lastPage": 3 }), 3, 100));
        assert!(!is_last_page(&json!({ "totalCount": 150 }), 1, 100));
        assert!(is_last_page(&json!({ "totalCount": 150 }), 2, 100));
        assert!(is_last_page(&json!({ "items": [1, 2] }), 1, 100));
    }

    /// A page as returned by `GET /sites/{siteId}/content-structures`
    /// (trimmed, but with Liferay's field names and envelope).
    const RECORDED_PAGE_2_OF_2: &str = r#"{
        "actions": {},
        "facets": [],
        "items": [
            {
                "availableLanguages": ["en-US"],
                "contentStructureFields": [
                    {
                        "dataType": "string",
                        "inputControl": "text",
                        "label": "Content",
                        "localizable": true,
                        "multiple": false,
                        "name": "content",
                        "nestedContentStructureFields": [],
                        "options": [],
                        "predefinedValue": "",
                        "repeatable": false,
                        "required": false,
                        "showLabel": true
                    }
                ],
                "dateCreated": "2026-01-14T10:02:11Z",
                "dateModified": "2026-01-14T10:02:11Z",
                "id": 32145,
                "name": "Event",
                "siteId": 20117
            }
        ],
        "lastPage": 2,
        "page": 2,
        "pageSize": 100,
        "totalCount": 101
    }"#;

    #[tokio::test]
    async fn recorded_page_fixture_is_parsed() {
        let page_2: Value = serde_json::from_str(RECORDED_PAGE_2_OF_2).unwrap();
        assert!(is_last_page(&page_2, 2, PAGE_SIZE));
        let mut site_pages = pages(many(1000, 100, "Structure"), PAGE_SIZE);
        site_pages[0]["lastPage"] = json!(2);
        site_pages[0]["totalCount"] = json!(101);
        site_pages.push(page_2);
        let api = FakeApi {
            site_pages: HashMap::from([(SITE, site_pages)]),
            ..Default::default()
        };
        let found = find_structure(&api, SITE, None, "Event").await.unwrap();
        assert_eq!(found["id"], 32145);
        assert_eq!(found["contentStructureFields"][0]["name"], "content");
    }

    #[tokio::test]
    async fn name_match_on_second_page() {
        let mut site = many(1000, 100, "Structure");
        site.push(structure(42, "Event"));
        let api = FakeApi {
            site_pages: HashMap::from([(SITE, pages(site, PAGE_SIZE))]),
            ..Default::default()
        };
        let found = find_structure(&api, SITE, None, "Event").await.unwrap();
        assert_eq!(found["id"], 42);
        assert_eq!(*api.page_requests.borrow(), vec![(SITE, 1), (SITE, 2)]);
    }

    #[tokio::test]
    async fn reads_every_page_before_giving_up_on_site() {
        let api = FakeApi {
            site_pages: HashMap::from([
                (SITE, pages(many(1000, 250, "Structure"), PAGE_SIZE)),
                (
                    GLOBAL,
                    pages(vec![structure(7, "Basic Web Content")], PAGE_SIZE),
                ),
            ]),
            global_id: Some(GLOBAL),
            ..Default::default()
        };
        let found = find_structure(&api, SITE, None, "Basic Web Content")
            .await
            .unwrap();
        assert_eq!(found["id"], 7);
        assert_eq!(
            *api.page_requests.borrow(),
            vec![(SITE, 1), (SITE, 2), (SITE, 3), (GLOBAL, 1)]
        );
    }

    #[tokio::test]
    async fn numeric_id_is_fetched_directly() {
        let api = FakeApi {
            by_id: HashMap::from([(36301, structure(36301, "Basic Web Content"))]),
            ..Default::default()
        };
        let found = find_structure(&api, SITE, None, "36301").await.unwrap();
        assert_eq!(found["name"], "Basic Web Content");
        assert!(api.page_requests.borrow().is_empty());
    }

    #[tokio::test]
    async fn unknown_numeric_id_falls_back_to_name_lookup() {
        let api = FakeApi {
            site_pages: HashMap::from([(SITE, pages(vec![structure(5, "2024")], PAGE_SIZE))]),
            global_id: Some(GLOBAL),
            ..Default::default()
        };
        let found = find_structure(&api, SITE, None, "2024").await.unwrap();
        assert_eq!(found["id"], 5);
    }

    #[tokio::test]
    async fn global_site_id_override_skips_resolution() {
        let api = FakeApi {
            site_pages: HashMap::from([(99, pages(vec![structure(7, "News")], PAGE_SIZE))]),
            global_id: None,
            ..Default::default()
        };
        let found = find_structure(&api, SITE, Some(99), "News").await.unwrap();
        assert_eq!(found["id"], 7);
    }

    #[tokio::test]
    async fn not_found_names_both_sites() {
        let api = FakeApi {
            site_pages: HashMap::from([
                (SITE, pages(many(1000, 30, "Structure"), PAGE_SIZE)),
                (
                    GLOBAL,
                    pages(vec![structure(7, "Basic Web Content")], PAGE_SIZE),
                ),
            ]),
            global_id: Some(GLOBAL),
            ..Default::default()
        };
        let err = find_structure(&api, SITE, None, "Missing")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Site ID 20117"), "{err}");
        assert!(err.contains("Global site (ID 20097)"), "{err}");
    }

    #[tokio::test]
    async fn global_site_resolution_failure_is_reported() {
        let api = FakeApi {
            site_pages: HashMap::from([(SITE, pages(vec![], PAGE_SIZE))]),
            global_id: None,
            ..Default::default()
        };
        let err = find_structure(&api, SITE, None, "Missing")
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Site ID 20117"), "{err}");
        assert!(err.contains("--global-site-id"), "{err}");
    }

    #[tokio::test]
    async fn global_site_not_searched_twice_when_target_is_global() {
        let api = FakeApi {
            site_pages: HashMap::from([(GLOBAL, pages(vec![], PAGE_SIZE))]),
            global_id: Some(GLOBAL),
            ..Default::default()
        };
        assert!(find_structure(&api, GLOBAL, None, "Missing").await.is_err());
        assert_eq!(*api.page_requests.borrow(), vec![(GLOBAL, 1)]);
    }
}
