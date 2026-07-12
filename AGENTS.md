# AGENTS.md

> **Read this before doing anything.** This document is the single source of truth for working in this repo. Do not invent files, do not invent behavior, do not change conventions without asking.

## 🔴 ABSOLUTE RULE: NEVER TOUCH GIT

**YOU ARE FORBIDDEN FROM RUNNING ANY GIT COMMAND.** This includes:
- `git add`, `git commit`, `git push`, `git pull`, `git merge`, `git rebase`
- `git checkout`, `git branch`, `git tag`, `git config`
- `git reset`, `git revert`, `git stash`, `git cherry-pick`

**YOU MAY ONLY SUGGEST THE EXACT COMMAND IN CHAT FOR THE USER TO COPY-PASTE AND RUN MANUALLY.**

> **Violation of this rule in past conversations caused branch corruption, lost commits, and extreme user frustration. Do not repeat this mistake.**

## 🔴 "GIVE" vs "ASSIGN"

- **"GIVE"** → **PRINT ONLY in chat.** Do NOT execute anything. Do NOT edit files. Do NOT run git. Just output text.
- **"ASSIGN"** → Execute the action (e.g. run `git commit`). But still: **NO GIT COMMANDS EVER** — only suggest commands for the user to run manually.

## Project: teivax

Rust + axum HTTP server. Serves anime character tag registries (Nikke, Arknights) as JSON.

### What it does

- Serves a static index of supported anime at `GET /`
- Returns tag arrays per anime at `GET /:id` (e.g. `GET /nikke`, `GET /arknights`)
- Dumps each anime's tag list to `json/{id}.json` on startup
- Exposes `/health`, `/loadavg`, `/metrics` for observability

### What it does NOT do

- It is not a scraper. It does not call external imageboards.
- It does not read TS files at runtime. TS files in `imageboards/` are reference data from a sibling project.
- It does not have a database. All data is compile-time const.

## Stack

| Concern | Crate | Version |
|---|---|---|
| HTTP server | `axum` | 0.7 |
| Runtime | `tokio` | 1 (`macros`, `rt-multi-thread`, `fs`) |
| Allocator | `mimalloc` | 0.1 (`extended` feature) |
| Async metrics | `axum-prometheus` | 0.7 |
| Metrics facade | `metrics` | 0.23 |
| System info | `sysinfo` | 0.32 |
| Serialization | `serde`, `serde_json` | 1 |
| Logging | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |
| HTTP middleware | `tower-http` | 0.5 (`cors`, `trace`) |
| Env loading | `dotenvy` | 0.15 |

## Build & Run

### Windows

```cmd
.\build.cmd
target\debug\teivax.exe
```

`build.cmd` sets MSVC env (`vcvars64.bat`), adds Git to PATH, then runs `cargo build %*`.

### Any platform

```bash
cargo build
./target/debug/teivax
```

### Env vars

| Var | Default | Notes |
|---|---|---|
| `PORT` | `3000` | TCP port to bind on `0.0.0.0` |
| `RUST_LOG` | `info` | tracing-subscriber EnvFilter |

`.env` is auto-bootstrapped from `.env.example` on first run by `env.rs::ensure_env_file()`. Do not call `dotenvy::dotenv()` directly; call `AppEnv::load()` instead.

## File Layout

```
teivax/
├── AGENTS.md              — this file
├── README.md              — user-facing docs (if any)
├── Cargo.toml             — dependencies, edition 2021
├── build.cmd              — Windows MSVC wrapper
├── .env.example           — env template (PORT, RUST_LOG)
├── .env                   — gitignored, auto-created
├── .gitignore             — see "Git" section
├── imageboards/           — reference TS data, NOT compiled (read-only)
│   ├── queryNikke.ts      — 106 nikke character tags
│   └── queryArknights.ts  — 77 arknights character tags
└── src/
    ├── main.rs            — tokio main, router setup, dump_json
    ├── env.rs             — AppEnv::load(), ensure_env_file
    ├── handlers.rs        — index, get_anime, health, loadavg
    ├── metrics.rs         — mimalloc global_allocator, system updater
    ├── data.rs            — Anime struct, REGISTRY const, by_id
    └── data/
        ├── nikke.rs       — TAGS: &[&str], 106 entries
        └── arknights.rs   — TAGS: &[&str], 77 entries
```

## Module Reference

### `src/main.rs`

- `#[tokio::main] async fn main()`
  1. `let env = AppEnv::load();` — loads `.env`, creates if missing
  2. `tracing_subscriber::fmt()` — init with `RUST_LOG` filter
  3. `metrics::describe_metrics()` — register Prometheus metric descriptions
  4. `metrics::spawn_system_updater()` — background task: CPU/mem/disk gauges
  5. `dump_json().await` — write `json/{id}.json` for each anime in REGISTRY
  6. Build `metrics_router` (no CORS) with `/metrics` + `/debug/mimalloc`
  7. Build `data_router` (CORS permissive) with `/`, `/:id`, `/health`, `/loadavg`
  8. `axum::serve()` on `0.0.0.0:{env.port}`

- `async fn dump_json()` — creates `json/`, iterates `REGISTRY`, writes `json/{id}.json` via `dump_one`. Errors are logged, not propagated.

- `async fn dump_one(path, items: &[&str]) -> io::Result<()>` — serializes items to pretty JSON, writes to disk, records `json_dump_duration_seconds` histogram.

### `src/env.rs`

- `pub struct AppEnv { pub port: u16 }`
- `pub fn AppEnv::load() -> Self`
  1. `ensure_env_file()` — if `.env` missing, copy from `.env.example`
  2. `dotenvy::dotenv()` — load vars from `.env` (no-op if missing)
  3. Parse `PORT` (fallback 3000)
- `fn ensure_env_file()` — copies `.env.example` to `.env` if `.env` does not exist; logs `created .env from .env.example`

### `src/handlers.rs`

- `pub struct AppState { pub started: Instant }` — cloned into axum state, used for uptime gauge
- `pub async fn index() -> Json` — returns `{"entries": [...], "endpoints": ["/health", "/metrics"]}`. Each entry: `{id, title, provider, tag_count, endpoint}`.
- `pub async fn health() -> &'static str` — returns `"ok"`
- `pub async fn get_anime(State, Path(id))` — looks up anime by id; 404 on miss, else returns `Json(a.tags)`. Records `anime_tags_count` gauge + `process_uptime_seconds` gauge.
- `pub async fn loadavg() -> Json<LoadAvg>` — returns `{one, five, fifteen}` from `sysinfo::System::load_average()`

### `src/data.rs`

```rust
pub mod nikke;
pub mod arknights;

#[derive(Debug, Clone)]
pub struct Anime {
    pub id: &'static str,
    pub title: &'static str,
    pub provider: &'static str,
    pub tags: &'static [&'static str],
}

pub const REGISTRY: &[Anime] = &[
    Anime { id: "nikke", title: "Nikke", provider: "rule34", tags: nikke::TAGS },
    Anime { id: "arknights", title: "Arknights", provider: "rule34", tags: arknights::TAGS },
];

pub fn by_id(id: &str) -> Option<&'static Anime> {
    REGISTRY.iter().find(|a| a.id == id)
}
```

### `src/data/{nikke,arknights}.rs`

```rust
pub const TAGS: &[&str] = &[
    "anis_(nikke)",
    "elegg_(nikke)",
    // ... 106 entries for nikke, 77 for arknights
];
```

### `src/metrics.rs`

- `#[global_allocator] static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;` — **only one place**. Do not duplicate.
- `pub fn describe_metrics()` — register all Prometheus metric descriptions
- `pub fn spawn_system_updater()` — background task updating system gauges every N seconds
- `pub fn record_json_dump(path: &str, dur: f64)` — records `json_dump_duration_seconds` histogram

## API Reference

### `GET /`

```json
{
  "entries": [
    {
      "id": "nikke",
      "title": "Nikke",
      "provider": "rule34",
      "tag_count": 106,
      "endpoint": "/nikke"
    },
    {
      "id": "arknights",
      "title": "Arknights",
      "provider": "rule34",
      "tag_count": 77,
      "endpoint": "/arknights"
    }
  ],
  "endpoints": ["/health", "/metrics"]
}
```

### `GET /:id`

Where `:id` is `nikke` or `arknights`. Returns `Json<&'static [&'static str]>`.

```json
["anis_(nikke)", "elegg_(nikke)", "rapi_(nikke)", ...]
```

Unknown id returns `404 Not Found` with body `unknown anime: <id>`.

### `GET /health`

Returns `ok` (text/plain).

### `GET /loadavg`

```json
{ "one": 0.5, "five": 0.7, "fifteen": 0.8 }
```

Windows: `sysinfo` returns zeros (no loadavg on Win32).

### `GET /metrics`

Prometheus text format. Includes HTTP request metrics + system gauges.

### `GET /debug/mimalloc`

Returns mimalloc stats as JSON. Server-side only (no CORS).

## Conventions

### Data
- **`&'static str` everywhere.** No `String` in REGISTRY. No runtime allocation for tag data.
- **New anime:**
  1. Create `src/data/<id>.rs` with `pub const TAGS: &[&str] = &[ ... ];`
  2. Add `pub mod <id>;` to `src/data.rs`
  3. Add `Anime { id, title, provider, tags: <id>::TAGS }` to `REGISTRY`
- **Provider** is a static label (e.g. `"rule34"`). It is not parsed, not derived, hardcoded. The actual imageboard is consumed by the sibling TS project.

### Style
- `use` statements grouped: std, then external crates, then `crate::`
- Snake case functions, PascalCase types, SCREAMING_SNAKE consts
- No `unwrap()` outside `main()` startup; use `?` or `expect("message")` at trust boundaries
- Errors logged via `tracing::warn!` / `tracing::error!`, not `eprintln!`
- No comments explaining what code does; only why

### Routing
- `data_router` — public endpoints, CORS permissive, tracing layer, prom layer, state
- `metrics_router` — server-side scrapers, no CORS
- Routes are flat; do not nest

### Git
- `Cargo.lock` **must** be committed (binary crate — reproducible builds)
- `.env` is gitignored
- `imageboards/*.ts` is **NOT** gitignored (kept for cross-reference). They are not read at build or runtime.
- `target/` is gitignored

## Gotchas

1. **Mimalloc global allocator lives in `metrics.rs` only.** Adding another `#[global_allocator]` in `main.rs` or elsewhere fails to compile. Do not move it.

2. **`include_str!` paths are relative to the source file containing the macro.** From `src/data.rs`, `../imageboards/foo.ts` resolves to `<project_root>/imageboards/foo.ts`. If you do not understand this, you will get `couldn't read ... (os error 3)`.

3. **The `imageboards/*.ts` files exist but are not compiled.** They are reference data. Do not `include_str!` them, do not `std::fs::read_to_string` them, do not generate code from them. The Rust data is hand-maintained in `src/data/*.rs`.

4. **`AppEnv::load()` must be called before any code reads env vars.** It both creates `.env` (if missing) and calls `dotenvy::dotenv()`. Calling `dotenvy::dotenv()` directly is wrong because `.env` may not exist yet.

5. **`Cargo.lock` must be committed.** Binary crate — ensures reproducible builds. Do not ignore it.

6. **Provider is a label, not a URL.** It describes which imageboard the sibling TS project consumes from. It is not a base URL, not parsed at runtime, not used to make HTTP calls.

7. **`dump_json()` is best-effort.** Failures to create `json/` or write files are logged at `warn` level and silently skipped. Do not change this to return errors unless the user asks.

## Don't

- Don't add `include_str!` for TS files.
- Don't add a `build.rs`.
- Don't change the `Anime` struct fields without updating `handlers.rs::index()`.
- Don't use `String` in REGISTRY — keep `&'static [&'static str]`.
- Don't add a `#[global_allocator]` anywhere except `metrics.rs`.
- Don't call `dotenvy::dotenv()` directly; use `AppEnv::load()`.
- Don't add `unwrap()` in request handlers.
- Don't add a database, cache, or external HTTP client.
- Don't add a `web framework` (Next, React, etc.) — this is a backend service.
- Don't move or rename files without updating this doc.

## Common Tasks

### Add a new anime

1. Create `src/data/bluearchive.rs`:
   ```rust
   pub const TAGS: &[&str] = &[
       "shiroko_(blue_archive)",
       "hoshino_(blue_archive)",
       // ...
   ];
   ```
2. Edit `src/data.rs`:
   ```rust
   pub mod bluearchive;
   // ...
   Anime { id: "bluearchive", title: "Blue Archive", provider: "rule34", tags: bluearchive::TAGS },
   ```
3. Build: `cargo build`
4. Verify: `curl http://127.0.0.1:3000/bluearchive | head -c 200`

### Change the port

Edit `.env` (or create from `.env.example`):
```
PORT=8080
```

Restart the server. Env is read once at startup.

### Update dependencies

Edit `Cargo.toml`, run `cargo build`. Lockfile regenerates.

### Inspect mimalloc stats

```bash
curl http://127.0.0.1:3000/debug/mimalloc | jq .
```

### Test locally

```powershell
# Windows
.\build.cmd
$env:PORT=4000
.\target\debug\teivax.exe
# In another shell:
Invoke-WebRequest http://127.0.0.1:4000/health
```

## Out of Scope

- Production deployment (Docker, k8s, systemd) — not in this repo
- Authentication, rate limiting, TLS — single-instance local service
- Hot reload — restart the process after data changes
- Multiple instances — REGISTRY is static, no shared state needed

## When in Doubt

1. Read the file. Do not guess.
2. If the file does not exist, **say so** before writing one.
3. If a behavior is not documented here, check the actual code. Do not extrapolate.
4. If the user asks for something that contradicts this doc, ask which wins.

## Anti-patterns (Do Not Repeat)

Lessons from past agent failures in this repo:

- **Do not delete or stub existing data.** `src/data/{nikke,arknights}.rs` are the real sources of truth. Replacing them with `include_str!("../imageboards/...ts")` is wrong because the TS files are external, may not exist, and may be incomplete. Always keep the Rust submodules intact.
- **Do not add `include_str!` for TS files** in this project. The TS data is reference-only, never read at build or runtime. If the user asks for the `provider` field, hardcode it as a string literal in `data.rs` — do not parse it from a file.
- **Do not modify `.gitignore` to ignore source files** that the build depends on. If `include_str!` references a file, that file must exist on disk regardless of git tracking. Adding a file to `.gitignore` does not delete it but signals the build is fragile.
- **Do not create `build.rs` to auto-generate code** unless the user explicitly asks. The data here is hand-maintained; auto-discovery is over-engineering.
- **When the user asks for a small change** (e.g. "add a `provider` field"), do the minimum: add the field, set a value, move on. Do not refactor the data layer, do not propose runtime loaders, do not regenerate from external files.
- **Trust boundaries matter.** `unwrap()`, `expect()`, and panics are acceptable in `main()` startup and module-level constants. Inside request handlers, propagate errors with `?` and let axum convert them to 5xx responses.
