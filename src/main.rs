use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use std::env;
use std::process::exit;

mod apprise;
mod grafana;
mod routes;
mod state;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let apprise_url = match apprise::get_apprise_url() {
        Some(h) => h,
        None => {
            log::error!("Invalid apprise host");
            exit(1);
        }
    };

    let app_state = state::AppState {
        apprise_url,
        append_labels: env::var("APPEND_LABELS").unwrap_or_else(|_| "true".to_string()) == "true",
        append_annotations: env::var("APPEND_ANNOTATIONS").unwrap_or_else(|_| "true".to_string())
            == "true",
        title_annotation: env::var("TITLE_ANNOTATION").ok(),
        body_annotation: env::var("BODY_ANNOTATION").ok(),
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(app_state.clone()))
            .route("/notify/{key}", web::post().to(routes::notify))
            .route("/health", web::get().to(routes::health))
    })
    .workers(utils::get_workers())
    .bind(format!("0.0.0.0:{}", utils::get_port()))?
    .run()
    .await
}
