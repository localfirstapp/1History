# Frontend Redesign Spec

Date: 2026-04-25

## Summary

Replace the current Bootstrap 2 + jQuery 1.11 + old ECharts (require.js) stack with a modern Vanilla JS + Vite + ECharts 5 + Tailwind CSS stack. No frontend framework. The redesign adds a KPI card row, dark/light theme switching, and a virtual-scroll search page.

## Decisions

| Topic | Decision |
|---|---|
| Tech stack | Vanilla JS + Vite + ECharts 5 + Tailwind CSS |
| Theme | Light + Dark, auto follows `prefers-color-scheme`, manual toggle stored in `localStorage` |
| Layout | Top navbar + KPI card row + charts |
| Search page | Virtual scroll (@tanstack/virtual) + client-side instant filter |
| Build | Manual two-step: `npm run build` then `cargo build` |

## Project Structure

```
1History/
├── frontend/               # new Vite project (replaces static/)
│   ├── src/
│   │   ├── main.js         # index page entry
│   │   ├── search.js       # search page entry
│   │   ├── details.js      # details page entry
│   │   ├── theme.js        # shared dark/light toggle
│   │   └── charts.js       # shared ECharts 5 wrappers
│   ├── pages/
│   │   ├── index.html
│   │   ├── search.html
│   │   └── details.html
│   ├── package.json
│   ├── vite.config.js      # multi-page build, output to dist/
│   └── tailwind.config.js
├── static/                 # deleted after migration
└── src/
    ├── database.rs         # add select_stats()
    └── web.rs              # RustEmbed folder → frontend/dist
```

`frontend/dist/` is gitignored. CI runs `npm run build` before `cargo build`.

## Pages

### Index (`/`)

**Navbar** (fixed top):
- Left: "1History" brand link
- Right: date range picker (flatpickr, replaces daterangepicker), search input, Search button, List button (shows count, opens `/search` in new tab)

**KPI card row** (4 cards, requires new `select_stats` backend query):
- Total Visits (in selected range)
- Unique Domains
- Active Days
- Today's Visits

**Charts row:**
- Daily PV line chart (full width, ECharts 5)
- Top 10 Titles pie + Top 10 Domains pie (side by side)
- Domain chart click → sets keyword to domain name and navigates

**Tables row:**
- TOP 100 by title + TOP 100 by domain (side by side)

### Search (`/search`)

**Navbar:** back to index link, current filter summary (date range + keyword if set)

**Toolbar:** client-side filter input (instant, no backend round-trip) + result count

**Virtual scroll list** (@tanstack/virtual, ~3KB):
- Columns: datetime, title/url (linked)
- Renders only visible rows; handles 20k+ records without lag

### Details (`/details/{ymd}`)

Layout and logic unchanged. CSS updated to match new theme.

## Theme System

CSS custom properties on `<html data-theme="light|dark">`:

```css
:root {
  --bg: #ffffff;
  --bg-card: #f8f9fa;
  --text: #1a1a1a;
  --text-muted: #6b7280;
  --border: #e5e7eb;
  --accent: #4A90D9;
}

[data-theme="dark"] {
  --bg: #0f1117;
  --bg-card: #1a1d2e;
  --text: #e2e8f0;
  --text-muted: #6b7280;
  --border: #2a2d3e;
  --accent: #6366f1;
}
```

`theme.js` initializes from `localStorage` → falls back to `prefers-color-scheme`. Toggle button in navbar top-right.

## Backend Changes

### `src/database.rs`

Add `select_stats(start: i64, end: i64) -> Result<Stats>`:

```rust
pub struct Stats {
    pub total_visits: i64,
    pub unique_domains: i64,
    pub active_days: i64,
    pub today_visits: i64,
}
```

SQL: four separate counts in one query using conditional aggregation.

### `src/web.rs`

1. `index` handler: call `select_stats`, pass `stats` to template context
2. Change `#[folder = "static"]` to `#[folder = "frontend/dist"]` on the `Asset` struct
3. No new routes needed

## Dependencies

**Frontend (`frontend/package.json`):**
```json
{
  "devDependencies": {
    "vite": "^6",
    "tailwindcss": "^4",
    "@tailwindcss/vite": "^4"
  },
  "dependencies": {
    "echarts": "^5",
    "flatpickr": "^4",
    "@tanstack/virtual-core": "^3"
  }
}
```

**Removed** (no longer vendored in `static/js/`): jQuery, Bootstrap JS/CSS, old ECharts, underscore, moment, daterangepicker.

## Build & Development

```bash
# Development
cd frontend && npm run dev   # Vite dev server on :5173, proxies /api to Rust on :9960
cargo run -- serve           # Rust server on :9960

# Production
cd frontend && npm run build  # outputs to frontend/dist/
cargo build --release         # embeds frontend/dist/ into binary
```

Vite config proxies `/details`, `/search`, `/` API calls to `localhost:9960` during development.

## Migration Steps

1. Create `frontend/` with Vite + Tailwind config
2. Port `index.html` → new layout with KPI cards
3. Port `search.html` → virtual scroll
4. Port `details.html` → updated styles only
5. Add `select_stats` to `database.rs`
6. Update `web.rs`: add stats to index context, change embed folder
7. Delete `static/` directory
8. Add `frontend/dist/` and `frontend/node_modules/` to `.gitignore`
9. Update CI: add `npm run build` step before `cargo build`

## What Is Not Changing

- All backend routes (`/`, `/details/{ymd}`, `/search`) — URLs unchanged
- Minijinja server-side templating for initial HTML render
- Database schema and all query logic (except the new `select_stats`)
- CLI interface and all non-web functionality
