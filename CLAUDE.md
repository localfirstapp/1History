# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Frontend (run from frontend/)
npm install          # install dependencies (first time)
npm run build        # build to static/dist/ (required before cargo build)
npm run dev          # dev server on :5173 with proxy to :9960

# Rust (requires frontend/static/dist/ to exist)
cargo build                                  # build
cargo test                                   # run all tests
cargo test <test_name>                       # run single test
cargo clippy --all-targets --all-features    # lint
cargo fmt -- --check                         # check formatting
cargo run -- serve                           # run server (http://127.0.0.1:9960)
cargo run -- show                            # show detected browser history files
```

Full workflow: `cd frontend && npm run build && cd .. && cargo run -- serve`

## Architecture

The app is a three-stage pipeline: **detect → backup → serve**.

### Modules

- **`source.rs`** — Opens browser SQLite files, detects browser type by querying known table names (`history_items` = Safari, `moz_historyvisits` = Firefox, `visits` = Chrome), and normalizes timestamps across three different browser epoch formats into Unix milliseconds.

- **`database.rs`** — The unified store (`onehistory.db`). Three tables: `onehistory_urls`, `onehistory_visits`, `import_records`. Deduplication is enforced via a UNIQUE constraint on `(item_id, visit_time)`. Batch inserts 100 rows per transaction.

- **`backup.rs`** — Orchestrates extraction: iterates source files, calls `source.rs` to read, writes to `database.rs`. Handles locked browser DBs by copying to a temp file first. Tracks import history in `import_records` to skip already-imported ranges.

- **`web.rs`** — Warp HTTP server. Routes: `/` (dashboard), `/details/{ymd}`, `/search`, `/db`, `/api/db/status`, `/api/backup` (POST), `/api/backup/{job_id}` (GET). HTML templates rendered with `minijinja`; JS/CSS served from `static/` (embedded via `rust-embed`). Frontend built with Vite outputs to `static/dist/`.

- **`util.rs`** — Browser profile path detection (globs for 16+ browser/OS combinations including Flatpak and snap variants). Also contains domain extraction and minijinja template filters.

- **`export.rs`** — Dumps visits to CSV (time, title, url, visit_type).

### Browser timestamp normalization

Each browser uses a different epoch in `source.rs`:
- Safari: NSDate (seconds since 2001-01-01)
- Firefox: PRTime (microseconds since Unix epoch)
- Chrome/Chromium: WebKit (microseconds since 1601-01-01)
