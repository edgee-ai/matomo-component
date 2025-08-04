// lib.rs
mod helpers;
use crate::exports::edgee::components::data_collection::{
    Data, Dict, EdgeeRequest, Event, Guest, HttpMethod, PageData, TrackData, UserData,
};
use helpers::*;
use std::collections::HashMap;
use url::form_urlencoded;

wit_bindgen::generate!({ world: "data-collection", path: ".edgee/wit", generate_all });
export!(Component);

struct Component;

pub struct Settings {
    pub site_id: String,
    pub endpoint_url: String,
    pub token_auth: Option<String>,
}

impl Settings {
    pub fn new(settings: Dict) -> anyhow::Result<Self> {
        let map = settings.into_iter().collect::<HashMap<_, _>>();
        Ok(Self {
            site_id: map.get("site_id").ok_or_else(|| anyhow::anyhow!("Missing site_id setting"))?,
            endpoint_url: map.get("endpoint_url").ok_or_else(|| anyhow::anyhow!("Missing endpoint_url setting")),
            token_auth: map.get("authentication_token").cloned(),
        })
    }
}

impl Guest for Component {
    fn page(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(p) = &event.data {
            common(&event, settings, Some(p), None, None)
        } else {
            Err("Expected page data".into())
        }
    }
    fn track(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(t) = &event.data {
            common(&event, settings, None, Some(t), None)
        } else {
            Err("Expected track data".into())
        }
    }
    fn user(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::User(u) = &event.data {
            common(&event, settings, None, None, Some(u))
        } else {
            Err("Expected user data".into())
        }
    }
}

fn common(
    event: &Event,
    settings_dict: Dict,
    page: Option<&PageData>,
    track: Option<&TrackData>,
    user: Option<&UserData>,
) -> Result<EdgeeRequest, String> {
    let settings = Settings::new(settings_dict).map_err(|e| e.to_string())?;
    let mut map: HashMap<String, String> = HashMap::new();
    let mut cvars: HashMap<String, String> = HashMap::new();
    let allow_sensitive = settings.token_auth.is_some();

    map.insert("idsite".into(), settings.site_id.clone());
    map.insert("rec".into(), "1".into());
    map.insert("apiv".into(), "1".into());
    map.insert("rand".into(), event.timestamp_millis.to_string());

    if let Some(page) = page {
        enrich_with_page_context(&mut map, page, &mut cvars);
    }
    if let Some(track) = track {
        enrich_with_track_context(&mut map, track, &mut cvars);
    }
    if let Some(user) = user {
        enrich_with_user_context(&mut map, user, &mut cvars);
    }

    enrich_with_client_context(&mut map, &event.context.client, &mut cvars, allow_sensitive);
    enrich_with_session_context(&mut map, &event.context.session, &mut cvars);
    enrich_with_campaign_context(&mut map, &event.context.campaign);

    if let Some(cv) = to_cvar(cvars) {
        map.insert("_cvar".into(), cv);
    }
    if let Some(token) = settings.token_auth {
        map.insert("token_auth".into(), token);
    }

    let qs = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(map.iter())
        .finish();

    let body = qs.clone();

    Ok(EdgeeRequest {
        method: HttpMethod::Post,
        url: format!("{}/matomo.php", settings.endpoint_url.trim_end_matches('/')),
        headers: vec![
            ("User-Agent".into(), "EdgeeComponent/1.0".into()),
            ("Accept".into(), "*/*".into()),
            (
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            ),
        ],
        forward_client_headers: false,
        body,
    })
}
