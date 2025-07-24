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
            site_id: map.get("site_id").cloned().unwrap_or_default(),
            endpoint_url: map.get("endpoint_url").cloned().unwrap_or_default(),
            token_auth: map.get("authentication_token").cloned(),
        })
    }
}

impl Guest for Component {
    fn page(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        match &event.data {
            Data::Page(p) => common(&event, settings, Some(p), None, None),
            _ => Err("Expected page data".into()),
        }
    }

    fn track(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        match &event.data {
            Data::Track(t) => common(&event, settings, None, Some(t), None),
            _ => Err("Expected track data".into()),
        }
    }

    fn user(event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        match &event.data {
            Data::User(u) => common(&event, settings, None, None, Some(u)),
            _ => Err("Expected user data".into()),
        }
    }
}

fn common(
    event: &Event,
    settings: Dict,
    page: Option<&PageData>,
    track: Option<&TrackData>,
    user: Option<&UserData>,
) -> Result<EdgeeRequest, String> {
    let settings = Settings::new(settings).map_err(|e| e.to_string())?;
    let mut map = HashMap::new();
    let mut cvars = HashMap::new();

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
    // common
    enrich_with_client_context(&mut map, &event.context.client);
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

    Ok(EdgeeRequest {
        method: HttpMethod::Get,
        url: format!(
            "{}/matomo.php?{}",
            settings.endpoint_url.trim_end_matches('/'),
            qs
        ),
        headers: vec![],
        forward_client_headers: true,
        body: "".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::components::data_collection::*;
    use uuid::Uuid;

    fn base_ctx() -> Context { /* same as before; include page, user, client, session, campaign */
    }

    #[test]
    fn page_includes_all() {
        let ctx = base_ctx();
        let page = Data::Page(ctx.page.clone());
        let event =
            Event { /* uuid, timestamps, event_type=Page, data=page, context=ctx, consent=None */ };
        let settings = vec![
            ("site_id".into(), "5".into()),
            ("endpoint_url".into(), "https://matomo.test".into()),
        ];
        let req = Component::page(event, settings).unwrap();
        let url = req.url;
        assert!(url.contains("action_name=Homepage"));
        assert!(url.contains("url=https%3A%2F%2Fexample.com"));
        assert!(url.contains("search="));
        assert!(url.contains("_cvar="));
    }

    #[test]
    fn track_includes_ea_ec_and_custom_props() {
        let data = Data::Track(TrackData {
            name: "Clicked".into(),
            properties: vec![("track_key".into(), "track_value".into())],
            products: vec![],
        });

        let mut event = sample_event(EventType::Track, data);
        event.context.user.anonymous_id = "abc123".into();

        let settings = vec![
            ("site_id".into(), "2".into()),
            ("endpoint_url".into(), "https://matomo.example.com/".into()),
        ];
        let req = Component::track(event, settings).unwrap();

        assert!(req.url.contains("e_a=Clicked"));
        assert!(req.url.contains("e_c=track"));
        assert!(req.url.contains("track_key"));
        assert!(req.url.contains("track_value"));
    }

    #[test]
    fn user_includes_uid_and_user_props() {
        let mut event = sample_event(
            EventType::User,
            Data::User(UserData {
                user_id: "test-user".into(),
                anonymous_id: "anon".into(),
                edgee_id: "eid".into(),
                properties: vec![("user_key".into(), "user_value".into())],
            }),
        );

        let settings = vec![
            ("site_id".into(), "2".into()),
            ("endpoint_url".into(), "https://matomo.example.com/".into()),
        ];
        let req = Component::user(event, settings).unwrap();

        assert!(req.url.contains("uid=test-user"));
        assert!(req.url.contains("user_key"));
        assert!(req.url.contains("user_value"));
    }
}
