// helpers.rs
use crate::exports::edgee::components::data_collection::{
    Campaign, Client, PageData, Session, TrackData, UserData,
};
use std::collections::HashMap;

pub fn insert_if_nonempty(map: &mut HashMap<String, String>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        map.insert(key.to_string(), value.to_string());
    }
}

pub fn to_cvar(vars: HashMap<String, String>) -> Option<String> {
    if vars.is_empty() {
        None
    } else {
        // Matomo _cvar expects JSON string: {"1":["key","value"],...}
        let mut cvars = serde_json::Map::new();
        let mut idx = 1;
        for (k, v) in vars {
            cvars.insert(idx.to_string(), serde_json::json!([k, v]));
            idx += 1;
        }
        Some(serde_json::to_string(&cvars).unwrap())
    }
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
    insert_if_nonempty(map, "_cvar", &page.category);
    // add custom page props
    for (k, v) in &page.properties {
        cvars.insert(format!("page_{}", k), v.clone());
    }
    if !page.keywords.is_empty() {
        cvars.insert(
            "page_keywords".into(),
            serde_json::to_string(&page.keywords).unwrap(),
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
    // Custom track props
    for (k, v) in &track.properties {
        cvars.insert(format!("track_{}", k), v.clone());
    }
}

pub fn enrich_with_user_context(
    map: &mut HashMap<String, String>,
    user: &UserData,
    cvars: &mut HashMap<String, String>,
) {
    if !user.user_id.trim().is_empty() {
        insert_if_nonempty(map, "uid", &user.user_id);
    }
    for (k, v) in &user.properties {
        cvars.insert(format!("user_{}", k), v.clone());
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
    map.insert("session_count".into(), session.session_count.to_string());
    cvars.insert("session_first_seen".into(), session.first_seen.to_string());
    cvars.insert("session_last_seen".into(), session.last_seen.to_string());
}

pub fn enrich_with_campaign_context(map: &mut HashMap<String, String>, campaign: &Campaign) {
    insert_if_nonempty(map, "_rcn", &campaign.name);
    insert_if_nonempty(map, "_rck", &campaign.term);
}

pub fn enrich_with_client_context(map: &mut HashMap<String, String>, client: &Client) {
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
    insert_if_nonempty(map, "model", &client.user_agent_model);
    insert_if_nonempty(map, "country", &client.country_code.to_lowercase());
    insert_if_nonempty(map, "region", &client.region);
    insert_if_nonempty(map, "city", &client.city);
}
