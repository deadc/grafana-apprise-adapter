use url::Url;

#[derive(Clone)]
pub struct AppState {
    pub apprise_url: Url,
    pub append_labels: bool,
    pub append_annotations: bool,
    pub title_annotation: Option<String>,
    pub body_annotation: Option<String>,
}
