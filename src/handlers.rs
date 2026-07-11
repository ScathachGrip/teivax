use std::time::Instant;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use metrics::gauge;
use serde::Serialize;
use sysinfo::{Pid, System};

use crate::data::{self, Anime, REGISTRY};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = "https://github.com/scathachgrip/teivax/blob/master/README.md#routing";

#[derive(Clone)]
pub struct AppState {
    pub started: Instant,
    pub server: String,
}

pub async fn root(State(state): State<AppState>) -> impl IntoResponse {
    let (rss, heap) = memory_stats();
    Json(serde_json::json!({
        "success": true,
        "message": "Hi, I'm alive!",
        "endpoint": REPO_URL,
        "date": chrono_format(),
        "rss": rss,
        "heap": heap,
        "server": state.server,
        "version": VERSION,
    }))
}

pub async fn index() -> impl IntoResponse {
    Json(serde_json::json!({
        "entries": REGISTRY.iter().map(|a| serde_json::json!({
            "id": a.id,
            "title": a.title,
            "provider": a.provider,
            "tag_count": a.tags.len(),
            "endpoint": format!("/{}", a.id),
        })).collect::<Vec<_>>(),
        "endpoints": ["/health", "/metrics", "/data"],
    }))
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn get_anime(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(a) = data::by_id(&id) else {
        return (StatusCode::NOT_FOUND, format!("unknown anime: {id}")).into_response();
    };
    record(a);
    gauge!("process_uptime_seconds").set(state.started.elapsed().as_secs_f64());
    (StatusCode::OK, Json(a.tags)).into_response()
}

fn record(a: &Anime) {
    gauge!("anime_tags_count", "anime" => a.id.to_string()).set(a.tags.len() as f64);
}

#[derive(Serialize)]
struct LoadAvg {
    one: f64,
    five: f64,
    fifteen: f64,
}

pub async fn loadavg() -> impl IntoResponse {
    let load = System::load_average();
    Json(LoadAvg {
        one: load.one,
        five: load.five,
        fifteen: load.fifteen,
    })
}

fn memory_stats() -> (String, String) {
    let pid = Pid::from_u32(std::process::id());
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);
    let rss = sys.process(pid).map(|p| p.memory()).unwrap_or(0) as f64 / 1024.0 / 1024.0;
    let (used_mb, total_mb) = mimalloc::MiMalloc::stats_json()
        .ok()
        .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v.to_bytes()).ok())
        .map(|v| {
            let used = v
                .get("committed")
                .and_then(|x| x.get("current"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let total = v
                .get("reserved")
                .and_then(|x| x.get("current"))
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            (used / 1024.0 / 1024.0, total / 1024.0 / 1024.0)
        })
        .unwrap_or((0.0, 0.0));
    (
        format!("{:.2} MB", rss),
        format!("{:.2}/{:.2} MB", used_mb, total_mb),
    )
}

fn chrono_format() -> String {
    use time::macros::format_description;
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let date_part = now
        .format(format_description!("[month]/[day]/[year]"))
        .unwrap_or_default();
    let time_part = now
        .format(format_description!("[hour repr:12]:[minute]:[second] [period]"))
        .unwrap_or_default();
    format!("{}, {}", date_part, time_part)
}
