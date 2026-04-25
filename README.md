# 1History

[![crates.io](https://img.shields.io/crates/v/onehistory.svg)](https://crates.io/crates/onehistory)
[![CI](https://github.com/localfirstapp/1History/actions/workflows/CI.yml/badge.svg)](https://github.com/localfirstapp/1History/actions/workflows/CI.yml)

> All your history in one file.

1History is a command line tool to backup your different browser histories into one file, and visualize them with a modern web UI.

[![ProductHunt](https://api.producthunt.com/widgets/embed-image/v1/review.svg?post_id=329191&theme=light)](https://www.producthunt.com/posts/1history?utm_source=badge-review&utm_medium=badge&utm_souce=badge-1history#discussion-body)

## Features

- Modern dashboard with dark/light theme, KPI cards, and interactive charts
- Search history across all browsers with case-insensitive filtering
- Virtual-scroll search results — handles 10k+ records without lag
- In-browser backup via the Database page (no CLI required after initial setup)
- Export as CSV file
- Entirely offline — no need to worry about privacy leaks
- Support Chrome/Firefox/Safari on macOS/Linux/Windows (including Flatpak and Snap variants)
- Well-designed schemas to avoid history duplication when backing up multiple times
- Single binary built in Rust 🦀

## Usage

```bash
onehistory 0.4.0

USAGE:
    onehistory [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -d, --db-file  <DB_FILE>    Database path [env: OH_DB_FILE=] [default: ~/onehistory.db]
    -h, --help                  Print help information
    -v, --verbose
    -V, --version               Print version information

SUBCOMMANDS:
    backup    Backup browser history to 1History
    export    Export history to CSV
    serve     Start HTTP server to visualize history
    show      Show default history files on your computer
```

### Backup

```bash
USAGE:
    onehistory backup [OPTIONS]

OPTIONS:
    -d, --disable-detect                    Disable auto detect history files
    -D, --dry-run
    -f, --history-files <HISTORY_FILES>     SQLite file path of different browsers
    -h, --help                              Print help information
```

`backup` is the main subcommand. It will auto-detect browser history files and import them into 1History.

You can also use `-f` to specify files manually. The history file naming convention:

| Browser | History Filename |
|---------|-----------------|
| Chrome  | History         |
| Firefox | places.sqlite   |
| Safari  | History.db      |

```bash
# -f can be used multiple times
# -d is useful when backing up while browsers are open
onehistory backup -d -f ~/some-dir/History.db -f ~/another-dir/places.sqlite
```

### Serve

After backing up, run `serve` to explore your history in the browser:

```bash
onehistory serve
# Open http://127.0.0.1:9960
```

The web UI includes:

- **Dashboard** — KPI cards, daily page view chart, top domains/pages
- **Search** — full-text search with virtual scroll and client-side instant filter
- **Database** — database status, per-browser breakdown, and in-browser backup

## Installation

### Homebrew

```bash
brew install 1History/onehistory/onehistory
```

### Binary

The [release page](https://github.com/localfirstapp/1History/releases) includes precompiled binaries for Linux, macOS and Windows.

### Cargo

```bash
cargo install onehistory
```

## Development

### Prerequisites

- Rust (stable)
- Node.js 18+

### Setup

```bash
git clone https://github.com/localfirstapp/1History
cd 1History

# Install frontend dependencies
cd frontend && npm install && cd ..
```

### Build

The frontend must be built before `cargo build`, as the output is embedded into the binary:

```bash
# Build frontend (outputs to static/dist/)
cd frontend && npm run build && cd ..

# Build binary
cargo build
```

### Running locally

```bash
# Terminal 1 — start Rust server
cargo run -- serve

# Terminal 2 — start Vite dev server with hot reload (proxies API to :9960)
cd frontend && npm run dev
# Open http://localhost:5173
```

Changes to files in `frontend/src/` are reflected immediately via Vite HMR. Changes to `static/*.html` (Minijinja templates) or Rust code require restarting the Rust server (and rebuilding if Rust code changed).

### Testing

```bash
cargo test
cargo clippy --all-targets --all-features
cargo fmt -- --check
```

## Changelog

See [CHANGELOG](CHANGELOG.md)

## FAQ

**`Error code 5: The database file is locked`**

This happens when your browser is open during backup, as SQLite only allows one connection at a time. Either close the browser first, or use `-d` with `-f` to point at a copy of the history file.

## LICENSE

Copyright (c) 2022 Jiacai Liu <dev@liujiacai.net>

1History is distributed under [GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.txt) license.
