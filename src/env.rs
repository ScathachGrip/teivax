use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub port: u16,
}

impl AppEnv {
    pub fn load() -> Self {
        ensure_env_file();
        let _ = dotenvy::dotenv();
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}

fn ensure_env_file() {
    let env = Path::new(".env");
    if env.exists() {
        return;
    }
    let example = Path::new(".env.schema");
    if !example.exists() {
        return;
    }
    if let Ok(src) = std::fs::read_to_string(example) {
        if std::fs::write(env, src).is_ok() {
            tracing::info!("created .env from .env.schema");
        }
    }
}
