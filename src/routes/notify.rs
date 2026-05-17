use awc::Client;
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::apprise;
use crate::grafana::GrafanaPayload;
use crate::state::AppState;

use serde::{Deserialize, Deserializer};

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multi(Vec<String>),
    }

    let result = StringOrVec::deserialize(deserializer)?;
    match result {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multi(v) => Ok(v),
    }
}

#[derive(Deserialize)]
pub struct NotifyQuery {
    pub append_labels: Option<bool>,
    pub append_annotations: Option<bool>,
    pub title_annotation: Option<String>,
    pub body_annotation: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub tag: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub tags: Vec<String>,
}

pub async fn notify(
    data: web::Json<GrafanaPayload>,
    key: web::Path<String>,
    state: web::Data<AppState>,
    query: web::Query<NotifyQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let config = apprise::NotificationConfig {
        append_labels: query.append_labels.unwrap_or(state.append_labels),
        append_annotations: query.append_annotations.unwrap_or(state.append_annotations),
        title_annotation: query
            .title_annotation
            .clone()
            .or_else(|| state.title_annotation.clone()),
        body_annotation: query
            .body_annotation
            .clone()
            .or_else(|| state.body_annotation.clone()),
    };
    let payload = apprise::ApprisePayload::from_grafana_payload(data.into_inner(), &config);
    let tags: Vec<String> = query
        .tag
        .iter()
        .chain(query.tags.iter())
        .cloned()
        .collect();
    let client = Client::default();
    let apprise_url = match apprise::get_apprise_notify_url(&state.apprise_url, &key, &tags) {
        Ok(url) => url,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let authorization_header = req.headers().get(header::AUTHORIZATION);

    let mut request = client.post(apprise_url.as_str());
    if let Some(header) = authorization_header {
        request = request.insert_header((header::AUTHORIZATION, header.clone()));
    }

    return match request.send_json(&payload).await {
        Ok(response) => HttpResponse::new(response.status()),
        Err(_) => HttpResponse::BadGateway().finish(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_urlencoded;

    #[test]
    fn test_deserialize_single_tag_string() {
        let query: NotifyQuery = serde_urlencoded::from_str("tags=telegram").unwrap();
        assert_eq!(query.tags, vec!["telegram".to_string()]);
    }

    #[test]
    fn test_deserialize_no_tags() {
        let query: NotifyQuery = serde_urlencoded::from_str("title_annotation=summary").unwrap();
        assert!(query.tags.is_empty());
    }

    #[test]
    fn test_deserialize_tag_param() {
        let query: NotifyQuery = serde_urlencoded::from_str("tag=matrix").unwrap();
        assert_eq!(query.tag, vec!["matrix".to_string()]);
    }

    #[test]
    fn test_deserialize_both_tag_and_tags() {
        let query: NotifyQuery =
            serde_urlencoded::from_str("tag=telegram&tags=matrix").unwrap();
        assert_eq!(query.tag, vec!["telegram".to_string()]);
        assert_eq!(query.tags, vec!["matrix".to_string()]);
    }
}
