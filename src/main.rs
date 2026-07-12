mod data;
mod env;
mod handlers;
mod metrics;

use std::net::SocketAddr;
use std::time::Instant;

use axum::routing::get;
use axum::response::Html;
use axum::Router;
use axum_prometheus::PrometheusMetricLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::data::{REGISTRY, BLOCKLISTS, GLOBAL_ANIME_GIRLS};
use crate::env::AppEnv;
use crate::handlers::AppState;

#[tokio::main]
async fn main() {
    let env = AppEnv::load();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    metrics::describe_metrics();
    metrics::spawn_system_updater();

    dump_json().await;

    let server = fetch_server();

    let state = AppState {
        started: Instant::now(),
        server,
    };

    let prom = PrometheusMetricLayer::pair();
    let (prom_layer, prom_handle) = prom;

    // Metrics router: NO CORS, server-side scraper only.
    let metrics_router = Router::new()
        .route("/metrics", get(move || {
            let h = prom_handle.clone();
            async move { h.render() }
        }))
        .route("/debug/mimalloc", get(|| async {
            match mimalloc::MiMalloc::stats_json() {
                Ok(s) => s.to_string_lossy().to_string().replace("\0", ""),
                Err(e) => format!("error: {e}"),
            }
        }));

    // Data router: permissive CORS for browsers.
    let data_router = Router::new()
        .route("/", get(handlers::root))
        .route("/data", get(handlers::index))
        .route("/:id", get(handlers::get_anime))
        .route("/global_anime_girls", get(handlers::get_global_anime_girls))
        .route("/blocklists", get(handlers::get_blocklists))
        .route("/health", get(handlers::health))
        .route("/loadavg", get(handlers::loadavg))
        .route("/playground", get(playground_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(prom_layer)
        .with_state(state);

    let app = Router::new()
        .merge(metrics_router)
        .merge(data_router);

    let addr = SocketAddr::from(([0, 0, 0, 0], env.port));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn dump_json() {
    use tokio::fs;
    if fs::create_dir_all("json").await.is_err() {
        tracing::warn!("failed to create json/ dir, skipping dump");
        return;
    }
    for anime in REGISTRY {
        let path = format!("json/{}.json", anime.id);
        let _ = dump_one(&path, anime.id, &anime.tags).await;
    }
    // dump global_anime_girls separately (different schema)
    let girls_path = "json/global_anime_girls.json";
    let start = std::time::Instant::now();
    if let Ok(json) = serde_json::to_string_pretty(GLOBAL_ANIME_GIRLS) {
        if tokio::fs::write(girls_path, json).await.is_ok() {
            metrics::record_json_dump(girls_path, start.elapsed().as_secs_f64());
        }
    }
    // dump blocklists (structured: array of {key, tags})
    let blk_path = "json/blocklists.json";
    let blk_start = std::time::Instant::now();
    if let Ok(blk_json) = serde_json::to_string_pretty(BLOCKLISTS) {
        if tokio::fs::write(blk_path, blk_json).await.is_ok() {
            metrics::record_json_dump(blk_path, blk_start.elapsed().as_secs_f64());
        }
    }
}

async fn dump_one(path: &str, id: &str, data: &data::TagData) -> std::io::Result<()> {
    let start = std::time::Instant::now();
    match data {
        data::TagData::Flat(items) => {
            let json = serde_json::to_string_pretty(items).expect("serialize");
            tokio::fs::write(path, json).await?;
        }
        data::TagData::Gif(items) => {
            let wrapper = match id {
                "data_gif" => serde_json::json!({"gif": items}),
                "data_gif_nsfw" => serde_json::json!({"nsfw": items}),
                _ => serde_json::to_value(items).expect("serialize"),
            };
            let json = serde_json::to_string_pretty(&wrapper).expect("serialize");
            tokio::fs::write(path, json).await?;
        }
    }
    metrics::record_json_dump(path, start.elapsed().as_secs_f64());
    Ok(())
}

fn fetch_server() -> String {
    let agent = ureq::AgentBuilder::new()
        .timeout_read(std::time::Duration::from_secs(3))
        .timeout_connect(std::time::Duration::from_secs(3))
        .build();
    match agent.get("https://ipwho.is/").call() {
        Ok(resp) => match resp.into_json::<serde_json::Value>() {
            Ok(v) => {
                let country = v.get("country").and_then(|x| x.as_str()).unwrap_or("").trim();
                let region = v.get("region").and_then(|x| x.as_str()).unwrap_or("").trim();
                if country.is_empty() || region.is_empty() {
                    "Local".to_string()
                } else {
                    format!("{country}, {region}")
                }
            }
            Err(_) => "Local".to_string(),
        },
        Err(_) => "Local".to_string(),
    }
}

async fn playground_handler() -> Html<String> {
    match tokio::fs::read_to_string("playground/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("Playground not generated. Run: cargo run --bin gen_playground".into()),
    }
}
