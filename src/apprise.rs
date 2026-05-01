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

impl From<GrafanaPayload> for ApprisePayload {
    fn from(gf_payload: GrafanaPayload) -> ApprisePayload {
        match gf_payload {
            GrafanaPayload::Legacy(p) => ApprisePayload::from(p),
            GrafanaPayload::Unified(p) => ApprisePayload::from(p),
        }
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

impl From<UnifiedPayload> for ApprisePayload {
    fn from(gf_payload: UnifiedPayload) -> ApprisePayload {
        let notification_type = match gf_payload.status.as_str() {
            "firing" => AppriseState::Failure,
            "resolved" => AppriseState::Success,
            _ => AppriseState::Info,
        };

        let mut body = gf_payload.message;

        if !gf_payload.common_labels.is_empty() {
            body.push_str("\n\nLabels:");
            for (key, value) in &gf_payload.common_labels {
                body.push_str(&format!("\n- {}: {}", key, value));
            }
        }

        if !gf_payload.common_annotations.is_empty() {
            body.push_str("\n\nAnnotations:");
            for (key, value) in &gf_payload.common_annotations {
                body.push_str(&format!("\n- {}: {}", key, value));
            }
        }

        ApprisePayload {
            title: gf_payload.title,
            body,
            notification_type,
        }
    }
}

pub fn get_apprise_notify_url(host: &Url, key: &str) -> Result<Url, ParseError> {
    host.join(&format!("/notify/{}", key))
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
            get_apprise_notify_url(&apprise_url, "foo")
                .unwrap()
                .as_str(),
            "http://apprise:8080/notify/foo"
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
}
