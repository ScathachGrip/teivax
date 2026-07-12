# teivax

Rust + axum HTTP server for anime character tag registries. Serves JSON tag arrays for multiple anime/game titles, plus system observability and a browser playground.

## Quick Start

```bash
cargo build
./target/debug/teivax
```

```powershell
# Windows
.\build.cmd
.\target\debug\teivax.exe
```

| Env        | Default | Notes                 |
| ---------- | ------- | --------------------- |
| `PORT`     | `3000`  | TCP bind on `0.0.0.0` |
| `RUST_LOG` | `info`  | tracing EnvFilter     |

`.env` auto-created from `.env.example` on first run.

## API

| Route                     | Returns | Description                                                      |
| ------------------------- | ------- | ---------------------------------------------------------------- |
| `GET /`                   | JSON    | Alive check — version, server location, memory stats             |
| `GET /data`               | JSON    | Registry index — all supported anime with tag counts             |
| `GET /:id`                | JSON    | Tag array for an anime (e.g. `/nikke`, `/arknights`, `/genshin`) |
| `GET /global_anime_girls` | JSON    | Global anime girls dataset                                       |
| `GET /blocklists`         | JSON    | Blocklist entries per category                                   |
| `GET /playground`         | HTML    | Browser UI (requires `gen_playground`)                           |
| `GET /health`             | text    | `"ok"`                                                           |
| `GET /loadavg`            | JSON    | System load average `{one, five, fifteen}`                       |
| `GET /metrics`            | text    | Prometheus metrics                                               |
| `GET /debug/mimalloc`     | JSON    | mimalloc allocator stats                                         |

### `GET /data`

```json
{
  "entries": [
    {
      "id": "nikke",
      "title": "Nikke",
      "provider": "rule34",
      "tag_count": 106,
      "endpoint": "/nikke"
    }
  ],
  "endpoints": [
    "/health",
    "/metrics",
    "/data",
    "/global_anime_girls",
    "/blocklists"
  ]
}
```

### `GET /:id`

Unknown ID returns `404 Not Found` with body `unknown anime: <id>`.

## Supported Titles (17)

| ID                   | Title                     | Provider | Tags              |
| -------------------- | ------------------------- | -------- | ----------------- |
| nikke                | Nikke                     | rule34   | 106               |
| arknights            | Arknights                 | rule34   | 77                |
| bluearchive          | Blue Archive              | rule34   | 112               |
| azurlane             | Azur Lane                 | rule34   | 82                |
| fgo                  | Fate/Grand Order          | rule34   | 153               |
| genshin              | Genshin Impact            | rule34   | 148               |
| genshin_danbooru     | Genshin Impact (Danbooru) | danbooru | 38                |
| honkai_starrail      | Honkai: Star Rail         | rule34   | 67                |
| girls_frontline      | Girls' Frontline          | rule34   | 48                |
| naruto               | Naruto                    | rule34   | 32                |
| bleach               | Bleach                    | rule34   | 33                |
| vtubers              | VTubers                   | rule34   | 376               |
| danbooru_sex         | Danbooru Sex Tags         | danbooru | 66                |
| gif_sex              | GIF Sex Tags              | others   | 32                |
| hentai_yandere       | Hentai Yandere Tags       | yandere  | 51                |
| ai_sex               | AI Sex Tags               | others   | 5                 |
| _global_anime_girls_ | —                         | —        | special dataset   |
| _blocklists_         | —                         | —        | blocklist entries |

## JSON Dumps

On startup, the server writes `json/{id}.json` for each anime and `json/global_anime_girls.json`, `json/blocklists.json`. Best-effort — failures logged, not fatal.

## Playground

```bash
cargo run --bin gen_playground
```

Generates `playground/index.html` — a browser UI for browsing tags. Served at `/playground`. Deployed to GitHub Pages via CI.

## Observability

Prometheus metrics at `/metrics`:

- **System**: CPU, memory, swap, loadavg, disk, network
- **Process**: CPU, RSS, virtual mem, threads, disk I/O, status
- **mimalloc**: commit, RSS, reserved, fragmentation, arenas, heaps, threads
- **App**: HTTP request count/duration, JSON dump timing, anime tag counts
- **Tokio**: worker thread count

## Architecture

```
                        ┌──────────────────────────────┐
                        │         HTTP Client          │
                        │   (curl / browser / bots)    │
                        └──────────────┬───────────────┘
                                       │
                                       │ GET /, /:id, /health
                                       │ GET /metrics, /loadavg
                                       v
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                     axum HTTP Server (0.7)                                       │
│                                                                                                  │
│  ┌──────────────────────────────────────┐          ┌──────────────────────────────────────────┐  │
│  │             data_router              │          │              metrics_router              │  │
│  │   (CORS permissive, tracing, prom)   │          │         (no CORS, direct scrape)         │  │
│  │                                      │          │                                          │  │
│  │   GET  /                             │          │   GET  /metrics                          │  │
│  │   GET  /:id                          │          │   GET  /debug/mimalloc                   │  │
│  │   GET  /health                       │          └──────────────────────┬───────────────────┘  │
│  │   GET  /loadavg                      │                                 │                      │
│  │   GET  /data                         │                                 v                      │
│  │   GET  /global_anime_girls           │          ┌──────────────────────────────────────────┐  │
│  │   GET  /blocklists                   │          │            prometheus metrics            │  │
│  │   GET  /playground                   │          │    (axum-prometheus + metrics 0.23)      │  │
│  └──────────────────┬───────────────────┘          └──────────────────────────────────────────┘  │
│                     │                                                                            │
│                     v                                                                            │
│  ┌──────────────────────────────────────┐                                                        │
│  │               handlers               │                                                        │
│  │   index() → REGISTRY                 │                                                        │
│  │   get_anime() → by_id()              │                                                        │
│  │   health() → "ok"                    │                                                        │
│  │   loadavg() → sysinfo::System        │                                                        │
│  └──────────────────┬───────────────────┘                                                        │
│                     │                                                                            │
│                     v                                                                            │
│  ┌──────────────────────────────────────┐                                                        │
│  │               data.rs                │                                                        │
│  │   Anime struct                       │                                                        │
│  │   REGISTRY: &[Anime]                 │                                                        │
│  │   by_id(id) → Option<&Anime>         │                                                        │
│  └──────────────────┬───────────────────┘                                                        │
│                     │                                                                            │
│           ┌─────────┴─────────┐                                                                  │
│           v                   v                                                                  │
│  ┌────────────────┐   ┌────────────────┐                                                         │
│  │    nikke.rs    │   │  arknights.rs  │  ... 16 more tag files                                  │
│  │    106 tags    │   │    77 tags     │                                                         │
│  └────────────────┘   └────────────────┘                                                         │
│                                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                               Background Tasks (tokio::spawn)                              │  │
│  │                                                                                            │  │
│  │  ┌────────────────────────────────────────┐    ┌────────────────────────────────────────┐  │  │
│  │  │            system_updater()            │    │               dump_json()              │  │  │
│  │  │  every N seconds:                      │    │  on startup:                           │  │  │
│  │  │  • sysinfo::System::* │    │  • write json/{id}.json                                 │  │  │
│  │  │  • CPU / mem / disk / net              │    │  • best-effort                         │  │  │
│  │  │  • process info / threads              │    └────────────────────────────────────────┘  │  │
│  │  └────────────────────────────────────────┘                                                │  │
│  └────────────────────────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────┬───────────────────────────────────────────────────┘
                                               │
                                               v
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                            Filesystem                                            │
│                                                                                                  │
│   json/                                                                                          │
│   ├── nikke.json                  <── dump_json() writes                                         │
│   ├── arknights.json                                                                             │
│   ├── genshin.json                                                                               │
│   ├── global_anime_girls.json                                                                    │
│   ├── blocklists.json                                                                            │
│   └── ...                                                                                        │
│                                                                                                  │
│   playground/                                                                                    │
│   └── index.html                  <── gen_playground binary                                      │
└──────────────────────────────────────────────┬───────────────────────────────────────────────────┘
                                               │
                                               v
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                       Global State (const)                                       │
│                                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                   REGISTRY (compile-time)                                  │  │
│  │  &[Anime; 17]                                                                              │  │
│  │  • id: &'static str                                                                        │  │
│  │  • title: &'static str                                                                     │  │
│  │  • provider: &'static str                                                                  │  │
│  │  • tags: &'static [&'static str]                                                           │  │
│  └────────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                                      AppState (runtime)                                    │  │
│  │  started: Instant                                                                          │  │
│  └────────────────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                             mimalloc::MiMalloc (global allocator)                          │  │
│  │  /debug/mimalloc exposes extended stats                                                    │  │
│  └────────────────────────────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Stack

| Concern         | Crate                            | Version    |
| --------------- | -------------------------------- | ---------- |
| HTTP server     | `axum`                           | 0.7        |
| Runtime         | `tokio`                          | 1          |
| Allocator       | `mimalloc` (extended)            | 0.1        |
| Metrics         | `axum-prometheus` + `metrics`    | 0.7 / 0.23 |
| System info     | `sysinfo`                        | 0.32       |
| Serialization   | `serde` + `serde_json`           | 1          |
| Logging         | `tracing` + `tracing-subscriber` | 0.1 / 0.3  |
| HTTP middleware | `tower-http` (cors, trace)       | 0.5        |
| Env loading     | `dotenvy`                        | 0.15       |

## CI/CD

### Release

Pushes to `master` trigger a build, tag creation (`v{VERSION}`), and GitHub Release with auto-generated changelog.

### Playground / GitHub Pages

Pushes to `master` generate the playground HTML and deploy to `gh-pages` branch.

## Adding a New Title

1. Create `src/data/my_anime.rs`:
   ```rust
   pub const TAGS: &[&str] = &[
       "character_name_(my_anime)",
   ];
   ```
2. Add `pub mod my_anime;` to `src/data.rs`
3. Add entry to `REGISTRY`:
   ```rust
   Anime { id: "my_anime", title: "My Anime", provider: "rule34", tags: my_anime::TAGS },
   ```
4. `cargo build`

## Known Issues

- `process_open_fds` always `0.0` — `open_files()` requires `sysinfo >= 0.33`, project pinned at `0.32`
- `system_cpu_brand` and `process_memory_used_bytes` gauges are described but not populated
- `loadavg` returns zeros on Windows (platform limitation)
- `imageboards/*.ts` are reference data only — not compiled, not read at runtime
