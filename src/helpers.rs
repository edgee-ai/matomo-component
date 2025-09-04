// helpers.rs
use crate::exports::edgee::components::data_collection::{
    Campaign, Client, PageData, Session, TrackData, UserData,
};
use serde_json::{json, Map as JsonMap};
use std::collections::HashMap;

pub fn insert_if_nonempty(map: &mut HashMap<String, String>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        map.insert(key.to_string(), value.to_string());
    }
}

/// Converts a map of custom properties into a Matomo-compatible `_cvar` JSON string.
///
/// Matomo supports up to 5 custom variables per event, encoded as:
/// `{ "1": ["key", "value"], "2": ["key", "value"], ... }`
///
/// - Filters out empty values
/// - Keeps only the first 5 entries (Matomo limit)
/// - Returns `None` if no valid entries remain
///
/// Used to enrich tracking requests with contextual metadata.
pub fn to_cvar(mut vars: HashMap<String, String>) -> Option<String> {
    vars.retain(|_, v| !v.trim().is_empty());
    if vars.is_empty() {
        return None;
    }
    let mut cvars = JsonMap::new();
    let mut idx = 1;
    for (k, v) in vars {
        cvars.insert(idx.to_string(), json!([k, v]));
        idx += 1;
        if idx > 5 {
            break;
        }
    }
    Some(serde_json::to_string(&cvars).unwrap())
}

pub fn enrich_with_page_context(
    map: &mut HashMap<String, String>,
    page: &PageData,
    cvars: &mut HashMap<String, String>,
) {
    insert_if_nonempty(map, "action_name", &page.title);
    insert_if_nonempty(map, "url", &page.url);
    insert_if_nonempty(map, "urlref", &page.referrer);
    insert_if_nonempty(map, "search", &page.search);
    insert_if_nonempty(map, "e_n", &page.name);
    insert_if_nonempty(map, "e_v", &page.path);
    insert_if_nonempty(map, "e_c", &page.category);

    for (k, v) in &page.properties {
        cvars.insert(format!("page_{k}"), v.clone());
    }

    if !page.keywords.is_empty() {
        cvars.insert(
            "page_keywords".into(),
            page.keywords.join(","),
        );
    }
}

pub fn enrich_with_track_context(
    map: &mut HashMap<String, String>,
    track: &TrackData,
    cvars: &mut HashMap<String, String>,
) {
    insert_if_nonempty(map, "e_a", &track.name);
    insert_if_nonempty(map, "e_c", "track");

    for (k, v) in &track.properties {
        match k.as_str() {
            "category" => {
                map.insert("e_c".to_string(), v.clone());
            }
            "label" => {
                map.insert("e_n".to_string(), v.clone());
            }
            "value" => {
                map.insert("e_v".to_string(), v.clone());
            }
            _ => {
                cvars.insert(format!("track_{k}"), v.clone());
            }
        }
    }

    if !track.products.is_empty() {
        let items: Vec<_> = track
            .products
            .iter()
            .map(|product| {
                let map: HashMap<_, _> = product.iter().cloned().collect();
                let sku = map.get("sku").cloned().unwrap_or_default();
                let name = map.get("name").cloned().unwrap_or_default();
                let category = map.get("category").cloned().unwrap_or_default();
                let price = map
                    .get("price")
                    .and_then(|p| p.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let quantity = map
                    .get("quantity")
                    .and_then(|q| q.parse::<i32>().ok())
                    .unwrap_or(1);
                json!([sku, name, category, price, quantity])
            })
            .collect();
        if let Ok(json) = serde_json::to_string(&items) {
            map.insert("ec_items".into(), json);
        }
    }
}

pub fn enrich_with_user_context(
    map: &mut HashMap<String, String>,
    user: &UserData,
    cvars: &mut HashMap<String, String>,
) {
    if !user.user_id.trim().is_empty() {
        insert_if_nonempty(map, "uid", &user.user_id);
    } else {
        // fallback cid : required by Matomo
        let cid = hex::encode(user.anonymous_id.clone())
            .chars()
            .take(16)
            .collect::<String>();
        map.insert("cid".into(), cid);
    }
    for (k, v) in &user.properties {
        cvars.insert(format!("user_{k}"), v.clone());
    }
}

pub fn enrich_with_session_context(
    map: &mut HashMap<String, String>,
    session: &Session,
    cvars: &mut HashMap<String, String>,
) {
    insert_if_nonempty(
        map,
        "new_visit",
        if session.session_start { "1" } else { "" },
    );
    insert_if_nonempty(map, "session_count", &session.session_count.to_string());
    cvars.insert("session_first_seen".into(), session.first_seen.to_string());
    cvars.insert("session_last_seen".into(), session.last_seen.to_string());
}

pub fn enrich_with_campaign_context(map: &mut HashMap<String, String>, campaign: &Campaign) {
    insert_if_nonempty(map, "_rcn", &campaign.name);
    insert_if_nonempty(map, "_rck", &campaign.term);
}

pub fn enrich_with_client_context(
    map: &mut HashMap<String, String>,
    client: &Client,
    cvars: &mut HashMap<String, String>,
    allow_sensitive: bool,
) {
    insert_if_nonempty(map, "ua", &client.user_agent);
    insert_if_nonempty(map, "lang", &client.locale);
    insert_if_nonempty(map, "timezone", &client.timezone);
    insert_if_nonempty(
        map,
        "res",
        &format!("{}x{}", client.screen_width, client.screen_height),
    );
    insert_if_nonempty(map, "os", &client.os_name);
    insert_if_nonempty(map, "os_version", &client.os_version);

    if !client.user_agent_model.trim().is_empty() {
        cvars.insert("client_model".into(), client.user_agent_model.clone());
    }

    if allow_sensitive {
        insert_if_nonempty(map, "country", &client.country_code.to_lowercase());
        insert_if_nonempty(map, "region", &client.region);
        insert_if_nonempty(map, "city", &client.city);
    } else {
        cvars.insert("client_country".into(), client.country_code.to_lowercase());
        cvars.insert("client_region".into(), client.region.clone());
        cvars.insert("client_city".into(), client.city.clone());
    }
}
