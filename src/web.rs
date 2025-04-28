use crate::cli::GpuModel;
use crate::scraper::{self, GpuListing, USER_AGENT};
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
use std::{net::SocketAddr, sync::Arc};
use tower_http::services::ServeDir;
use anyhow::anyhow;

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
        // Errors fetching/parsing a single model result in an empty list for that model,
        // allowing the page to still load with data from other models.
        match fetch_and_parse(&model_url).await {
            Ok(listings) => Ok::<(GpuModel, Vec<GpuListing>), anyhow::Error>((*model, listings)),
            Err(e) => {
                // Log the error server-side but don't fail the whole request
                eprintln!("Failed to fetch/parse listings for {:?}: {}", model, e);
                Ok::<(GpuModel, Vec<GpuListing>), anyhow::Error>((*model, Vec::new()))
            },
        }
    });
    let results: Vec<Result<(GpuModel, Vec<GpuListing>), _>> = join_all(fetch_futures).await;
    let all_listings: Vec<GpuListing> = results
        .into_iter()
        .filter_map(|res| res.ok()) // Filter out errors here
        .flat_map(|(_, listings)| listings)
        .collect();
    let template = IndexTemplate {
        title: "All GPU Listings".to_string(),
        listings: all_listings,
        models: models_to_check.to_vec(),
        current_model: None,
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    // Render template or return error string
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template rendering failed: {}", e),
        )
            .into_response(),
    }
}

// Handler for individual GPU model pages
async fn gpu_model_handler(
    State(_state): State<Arc<AppState>>,
    Path(model_str): Path<String>,
) -> Result<Html<String>, AppError> { // Return Result using AppError
    let model: GpuModel = model_str.parse()
        // Use map_err to convert the parsing error into AppError
        .map_err(|_| AppError(anyhow!("Invalid GPU model specified: {}", model_str)))?;
    let model_url = format!("{}{}", scraper::BASE_URL, model);
    // Use `?` to propagate errors from fetch_and_parse, automatically converting them to AppError
    let listings = fetch_and_parse(&model_url).await?;
    let template = IndexTemplate {
        title: format!("{:?} Listings", model), // Use Debug format for enum
        listings,
        models: GpuModel::value_variants().to_vec(),
        current_model: Some(model),
        last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    // Render the template, converting template errors into AppError using `?`
    let html_output = template.render()
        .map_err(|e| AppError(anyhow!(e).context("Template rendering failed")))?;
    Ok(Html(html_output)) // Return Ok(Html(...)) on success
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
        .danger_accept_invalid_certs(true)
        .http1_only()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(15))
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
    use chrono::Local;
    use axum::extract::ConnectInfo;
    use std::net::SocketAddr as StdSocketAddr;
    println!("Listening on http://{}", listen_addr);
    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/", get(home_handler))
        .route("/gpu/:model", get(gpu_model_handler))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .with_state(state)
        .layer(axum::middleware::from_fn(|req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
            // Logging middleware: log timestamp, IP, method, path
            let method = req.method().clone();
            let path = req.uri().path().to_string();
            let remote_addr = req.extensions().get::<ConnectInfo<StdSocketAddr>>()
                .map(|info| info.0.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let now = Local::now();
            println!("[{}] {} {} {}", now.format("%Y-%m-%d %H:%M:%S"), remote_addr, method, path);
            next.run(req)
        }));
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<StdSocketAddr>())
        .await
        .context("Web server failed")?;
    Ok(())
}
