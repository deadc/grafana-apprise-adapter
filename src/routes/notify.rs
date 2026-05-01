use awc::Client;
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::apprise;
use crate::grafana::GrafanaPayload;
use crate::state::AppState;

pub async fn notify(
    data: web::Json<GrafanaPayload>,
    key: web::Path<String>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let payload = apprise::ApprisePayload::from(data.into_inner());
    let client = Client::default();
    let apprise_url = match apprise::get_apprise_notify_url(&state.apprise_url, &key) {
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
