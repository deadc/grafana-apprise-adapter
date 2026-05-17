use serde::Serialize;
use std::env;
use url::{ParseError, Url};

use crate::grafana::{GrafanaPayload, GrafanaState, LegacyPayload, UnifiedPayload};

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AppriseState {
    Info,
    Success,
    Warning,
    Failure,
}

impl From<GrafanaState> for AppriseState {
    fn from(gf_state: GrafanaState) -> AppriseState {
        match gf_state {
            GrafanaState::Ok => AppriseState::Success,
            GrafanaState::Paused => AppriseState::Info,
            GrafanaState::Alerting => AppriseState::Failure,
            GrafanaState::Pending => AppriseState::Info,
            GrafanaState::NoData => AppriseState::Warning,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ApprisePayload {
    pub title: String,
    pub body: String,

    #[serde(rename = "type")]
    pub notification_type: AppriseState,
}

pub struct NotificationConfig {
    pub append_labels: bool,
    pub append_annotations: bool,
    pub title_annotation: Option<String>,
    pub body_annotation: Option<String>,
}

impl ApprisePayload {
    pub fn from_grafana_payload(gf_payload: GrafanaPayload, config: &NotificationConfig) -> Self {
        match gf_payload {
            GrafanaPayload::Legacy(p) => ApprisePayload::from(p),
            GrafanaPayload::Unified(p) => ApprisePayload::from_unified(p, config),
        }
    }

    fn from_unified(gf_payload: UnifiedPayload, config: &NotificationConfig) -> Self {
        let notification_type = match gf_payload.status.as_str() {
            "firing" => AppriseState::Failure,
            "resolved" => AppriseState::Success,
            _ => AppriseState::Info,
        };

        let title = config
            .title_annotation
            .as_ref()
            .and_then(|key| gf_payload.common_annotations.get(key).cloned())
            .unwrap_or_else(|| gf_payload.title.clone());

        let mut body = config
            .body_annotation
            .as_ref()
            .and_then(|key| gf_payload.common_annotations.get(key).cloned())
            .unwrap_or_else(|| gf_payload.message.clone());

        if config.append_labels && !gf_payload.common_labels.is_empty() {
            body.push_str("\n\nLabels:");
            for (key, value) in &gf_payload.common_labels {
                body.push_str(&format!("\n- {}: {}", key, value));
            }
        }

        if config.append_annotations && !gf_payload.common_annotations.is_empty() {
            body.push_str("\n\nAnnotations:");
            for (key, value) in &gf_payload.common_annotations {
                body.push_str(&format!("\n- {}: {}", key, value));
            }
        }

        ApprisePayload {
            title,
            body,
            notification_type,
        }
    }
}

impl From<UnifiedPayload> for ApprisePayload {
    fn from(gf_payload: UnifiedPayload) -> ApprisePayload {
        ApprisePayload::from_unified(
            gf_payload,
            &NotificationConfig {
                append_labels: true,
                append_annotations: true,
                title_annotation: None,
                body_annotation: None,
            },
        )
    }
}

impl From<GrafanaPayload> for ApprisePayload {
    fn from(gf_payload: GrafanaPayload) -> ApprisePayload {
        ApprisePayload::from_grafana_payload(
            gf_payload,
            &NotificationConfig {
                append_labels: true,
                append_annotations: true,
                title_annotation: None,
                body_annotation: None,
            },
        )
    }
}

impl From<LegacyPayload> for ApprisePayload {
    fn from(gf_payload: LegacyPayload) -> ApprisePayload {
        ApprisePayload {
            title: gf_payload.title,
            body: gf_payload.message,
            notification_type: AppriseState::from(gf_payload.state),
        }
    }
}

pub fn get_apprise_notify_url(host: &Url, key: &str, tags: &[String]) -> Result<Url, ParseError> {
    let mut url = host.join(&format!("/notify/{}", key))?;
    if !tags.is_empty() {
        url.set_query(Some(&format!("tag={}", tags.join(","))));
    }
    Ok(url)
}

pub fn get_apprise_url() -> Option<Url> {
    let apprise_env = env::var("APPRISE_URL").ok()?;
    Url::parse(&apprise_env).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env::set_var;
    use url::Url;

    #[test]
    fn test_get_apprise_url() {
        assert!(get_apprise_url().is_none());
        set_var("APPRISE_URL", "http://apprise:8080");
        let parsed_url = get_apprise_url().unwrap();
        assert_eq!(parsed_url.domain(), Some("apprise"));
        assert_eq!(parsed_url.port(), Some(8080));
        assert_eq!(parsed_url.scheme(), "http");
    }

    #[test]
    fn test_get_apprise_notify_url() {
        let apprise_url = Url::parse("http://apprise:8080").unwrap();
        assert_eq!(
            get_apprise_notify_url(&apprise_url, "foo", &[])
                .unwrap()
                .as_str(),
            "http://apprise:8080/notify/foo"
        );
    }

    #[test]
    fn test_get_apprise_notify_url_with_tags() {
        let apprise_url = Url::parse("http://apprise:8080").unwrap();
        assert_eq!(
            get_apprise_notify_url(&apprise_url, "foo", &["tag1".to_string(), "tag2".to_string()])
                .unwrap()
                .as_str(),
            "http://apprise:8080/notify/foo?tag=tag1,tag2"
        );
    }

    #[test]
    fn test_get_apprise_notify_url_with_single_tag() {
        let apprise_url = Url::parse("http://apprise:8080").unwrap();
        assert_eq!(
            get_apprise_notify_url(&apprise_url, "bar", &["mytag".to_string()])
                .unwrap()
                .as_str(),
            "http://apprise:8080/notify/bar?tag=mytag"
        );
    }

    #[test]
    fn test_unified_payload_conversion() {
        let mut common_labels = HashMap::new();
        common_labels.insert("severity".to_string(), "critical".to_string());

        let payload = UnifiedPayload {
            title: "Test Alert".to_string(),
            message: "Something is wrong".to_string(),
            status: "firing".to_string(),
            common_labels,
            common_annotations: HashMap::new(),
        };
        let apprise_payload = ApprisePayload::from(payload);
        assert_eq!(apprise_payload.title, "Test Alert");
        assert!(apprise_payload.body.contains("Something is wrong"));
        assert!(apprise_payload.body.contains("Labels:"));
        assert!(apprise_payload.body.contains("- severity: critical"));
        assert!(matches!(
            apprise_payload.notification_type,
            AppriseState::Failure
        ));

        let payload_resolved = UnifiedPayload {
            title: "Test Alert".to_string(),
            message: "Everything is fine".to_string(),
            status: "resolved".to_string(),
            common_labels: HashMap::new(),
            common_annotations: HashMap::new(),
        };
        let apprise_payload_resolved = ApprisePayload::from(payload_resolved);
        assert!(matches!(
            apprise_payload_resolved.notification_type,
            AppriseState::Success
        ));
    }

    #[test]
    fn test_unified_payload_config_overrides() {
        let mut common_labels = HashMap::new();
        common_labels.insert("severity".to_string(), "critical".to_string());
        let mut common_annotations = HashMap::new();
        common_annotations.insert("description".to_string(), "CUSTOM BODY".to_string());
        common_annotations.insert("summary".to_string(), "CUSTOM TITLE".to_string());

        let payload = UnifiedPayload {
            title: "Original Title".to_string(),
            message: "Original Message".to_string(),
            status: "firing".to_string(),
            common_labels,
            common_annotations,
        };

        // Test skipping labels and annotations
        let config_skip = NotificationConfig {
            append_labels: false,
            append_annotations: false,
            title_annotation: None,
            body_annotation: None,
        };
        let apprise_payload_skip = ApprisePayload::from_unified(payload.clone(), &config_skip);
        assert_eq!(apprise_payload_skip.title, "Original Title");
        assert_eq!(apprise_payload_skip.body, "Original Message");

        // Test using specific annotations
        let config_annotations = NotificationConfig {
            append_labels: false,
            append_annotations: false,
            title_annotation: Some("summary".to_string()),
            body_annotation: Some("description".to_string()),
        };
        let apprise_payload_anno = ApprisePayload::from_unified(payload, &config_annotations);
        assert_eq!(apprise_payload_anno.title, "CUSTOM TITLE");
        assert_eq!(apprise_payload_anno.body, "CUSTOM BODY");
    }
}
