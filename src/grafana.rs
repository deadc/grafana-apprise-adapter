use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GrafanaState {
    Ok,
    Paused,
    Alerting,
    Pending,
    NoData,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GrafanaPayload {
    Legacy(LegacyPayload),
    Unified(UnifiedPayload),
}

#[derive(Deserialize, Debug, Clone)]
pub struct LegacyPayload {
    pub title: String,
    pub message: String,
    pub state: GrafanaState,

    #[serde(rename = "imageUrl")]
    pub image: String,

    pub tags: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UnifiedPayload {
    pub title: String,
    pub message: String,
    pub status: String,

    #[serde(default, rename = "commonLabels")]
    pub common_labels: HashMap<String, String>,

    #[serde(default, rename = "commonAnnotations")]
    pub common_annotations: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_legacy() {
        let json = r#"{
            "title": "Legacy Alert",
            "message": "Old format",
            "state": "alerting",
            "imageUrl": "http://example.com/image.png",
            "tags": {"foo": "bar"}
        }"#;
        let payload: GrafanaPayload = serde_json::from_str(json).unwrap();
        match payload {
            GrafanaPayload::Legacy(p) => {
                assert_eq!(p.title, "Legacy Alert");
                assert!(matches!(p.state, GrafanaState::Alerting));
            }
            _ => panic!("Expected Legacy payload"),
        }
    }

    #[test]
    fn test_deserialize_unified() {
        let json = r#"{
            "title": "Unified Alert",
            "message": "Grafana 11 format",
            "status": "firing",
            "commonLabels": {"severity": "critical"}
        }"#;
        let payload: GrafanaPayload = serde_json::from_str(json).unwrap();
        match payload {
            GrafanaPayload::Unified(p) => {
                assert_eq!(p.title, "Unified Alert");
                assert_eq!(p.status, "firing");
                assert_eq!(p.common_labels.get("severity").unwrap(), "critical");
            }
            _ => panic!("Expected Unified payload"),
        }
    }
}
