use crate::cli::{GpuModel, ParseGpuModelError};
use crate::scraper::{self, GpuListing};
use anyhow::{Context, Result};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use clap::ValueEnum;
use futures::future::join_all;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    listings: Vec<GpuListing>,
    models: Vec<GpuModel>,
    current_model: Option<GpuModel>,
    last_updated: String,
}

#[derive(Clone)]
struct AppState {}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Handler for the home page (all GPUs)
async fn home_handler(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let models_to_check = GpuModel::value_variants();
    let fetch_futures = models_to_check.iter().map(|model| async move {
        let model_url = format!("{}{}", scraper::BASE_URL, model);
        match fetch_and_parse(&model_url).await {
            Ok(listings) => Ok::<(GpuModel, Vec<GpuListing>), anyhow::Error>((*model, listings)),
            Err(_) => Ok::<(GpuModel, Vec<GpuListing>), anyhow::Error>((*model, Vec::new())),
        }
    });
    let results: Vec<Result<(GpuModel, Vec<GpuListing>), _>> = join_all(fetch_futures).await;
    let all_listings: Vec<GpuListing> = results
        .into_iter()
        .filter_map(|res: Result<(GpuModel, Vec<GpuListing>), _>| res.ok())
        .flat_map(|(_, listings)| listings)
        .collect();
    let template = IndexTemplate {
        title: "All GPU Listings".to_string(),
        listings: all_listings,
        models: models_to_check.to_vec(),
        current_model: None,
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    Html(template.render().unwrap_or_else(|_| "Template error".to_string()))
}

// Handler for individual GPU model pages
async fn gpu_model_handler(
    State(_state): State<Arc<AppState>>,
    Path(model_str): Path<String>,
) -> impl IntoResponse {
    let model: GpuModel = match model_str.parse() {
        Ok(m) => m,
        Err(_) => return Html("Invalid GPU model".to_string()),
    };
    let model_url = format!("{}{}", scraper::BASE_URL, model);
    let listings = fetch_and_parse(&model_url).await.unwrap_or_default();
    let template = IndexTemplate {
        title: format!("{:?} Listings", model),
        listings,
        models: GpuModel::value_variants().to_vec(),
        current_model: Some(model),
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    Html(template.render().unwrap_or_else(|_| "Template error".to_string()))
}

async fn fetch_and_parse(url: &str) -> Result<Vec<GpuListing>> {
    let html = fetch_html(url, false).await?;
    scraper::parse_listings(&html, false)
}

pub async fn fetch_html(url: &str, quiet: bool) -> Result<String> {
    if !quiet {
        println!("Fetching URL: {}", url);
    }
    let client = reqwest::Client::builder()
        .user_agent(scraper::USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()?;

    let response = client.get(url).send().await.context("Failed to send request")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Request failed with status: {} for URL: {}",
            response.status(),
            url
        );
    }

    response.text().await.context("Failed to read response text")
}

pub async fn run_server(listen_addr: SocketAddr) -> Result<()> {
    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/", get(home_handler))
        .route("/gpu/:model", get(gpu_model_handler))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app.into_make_service())
        .await
        .context("Web server failed")?;
    Ok(())
}
