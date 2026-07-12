<div align="center">
<h4 align="center">A high-performance decentralized for custom branding registry.</h4>
<p align="center">
	<a href="https://github.com/scathachgrip/teivax/actions/workflows/playground.yml"><img src="https://github.com/ScathachGrip/teivax/actions/workflows/playground.yml/badge.svg"></a>
	<a href="https://qlty.sh/gh/ScathachGrip/projects/teivax"><img src="https://qlty.sh/gh/ScathachGrip/projects/teivax/maintainability.svg" alt="Maintainability" /></a>
</p>

Powered by the axum web framework for async concurrency without garbage collection overhead. Exposes first-class Prometheus observability, mimalloc-accelerated allocation, and a compile-time static registry that never touches a database — just pure `&'static [&'static str]`.

<a href="https://scathachgrip.github.io/teivax">Playground</a> •
<a href="https://github.com/scathachgrip/teivax/blob/master/CONTRIBUTING.md">Contributing</a> •
<a href="https://github.com/scathachgrip/teivax/issues/new/choose">Report Issues</a>

</div>

---

<a href="https://scathachgrip.github.io/teivax/"><img align="right" src="src/resources/project/images/teivax.png" width="300"></a>

- [ScathachGrip/teivax](#)
  - [The problem](#the-problem)
  - [The solution](#the-solution)
  - [Architecture](#architecture)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
    - [Installation](#installation)
      - [Docker](#docker)
      - [Manual](#manual)
    - [Tests](#tests)
    - [Nhentai Guide](#nhentai-guide)
  - [Playground](https://sinkaroid.github.io/jandapress)
    - [Routing](#playground)
    - [Status response](#status-response)
    - [Observability](#observability)
    - [Stack](#stack)
  - [Adding a new Data](#adding-a-new-data)
  - [Pronunciation](#Pronunciation)
  - [Client libraries](#client-libraries)
  - [Legal](#legal)

## The Problems

Managing tag registries across multiple Discord bots is a maintenance nightmare. When a new anime character drops, every bot project must be edited individually — update the list, rebuild, redeploy. Teams maintaining 5, 10, or 20 bots end up copy-pasting the same 100+ character array across repos, creating drift between deployments. One bot gets `"anis_(nikke)"` with correct formatting, another uses a stale alias. There is no single source of truth, no centralized versioning, and no way to propagate updates without touching each codebase by hand.

## The Solutions

teivax decouples the tag registry from your bot code. Instead of embedding character lists in every project, point all your bots at a single HTTP endpoint. Add or update tags in one place — teivax — and every bot picks up the change on its next request. The registry is compile-time constant, served with sub-millisecond latency, and needs no database, no cache warming, and no runtime synchronization. One `cargo build` propagates new characters to every consumer.

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
│  │  │  • sysinfo::System::*                  │    │  • write json/{id}.json                │  │  │
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
| genshin              | Genshin Impact            | rule34   | 148               |
| nikke                | Nikke                     | rule34   | 106               |
| arknights            | Arknights                 | rule34   | 77                |
| bluearchive          | Blue Archive              | rule34   | 112               |
| azurlane             | Azur Lane                 | rule34   | 82                |
| fgo                  | Fate/Grand Order          | rule34   | 153               |
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

## Adding a New Data

1. Create `src/data/my_config.rs`:
   ```rust
   pub const TAGS: &[&str] = &[
       "character_name_(my_config)",
   ];
   ```
2. Add `pub mod my_config;` to `src/data.rs`
3. Add entry to `REGISTRY`:
   ```rust
   Anime { id: "my_config", title: "my_config", provider: "rule34", tags: my_config::TAGS },
   ```
4. `cargo build`

## Known Issues

- `process_open_fds` always `0.0` — `open_files()` requires `sysinfo >= 0.33`, project pinned at `0.32`
- `system_cpu_brand` and `process_memory_used_bytes` gauges are described but not populated
- `loadavg` returns zeros on Windows (platform limitation)
- `imageboards/*.ts`, `custom_bot/client/*.ts` are reference data only — not compiled, not read at runtime

## Pronunciation

[`en_US`](https://www.localeplanet.com/java/en-US/index.html) • **/tay·vaks/** — **Tei** (**Teyvat**, Genshin reference; reflects the project's primary data source) + **Ax** (shorthand for **Axum**, the Rust web framework).

## Legal

This tool can be freely copied, modified, altered, distributed without any attribution whatsoever. However, if you feel
like this tool deserves an attribution, mention it. It won't hurt anybody.

> Licence: WTF.
