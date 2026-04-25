# Frontend Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Bootstrap 2 + jQuery 1.11 + old ECharts stack with Vanilla JS + Vite + ECharts 5 + Tailwind CSS, adding a KPI card row, dark/light theme, virtual-scroll search page, and a new DB management page with async backup.

**Architecture:** Vite multi-page app in `frontend/` outputs to `frontend/dist/`, which is embedded into the Rust binary via `rust-embed`. Minijinja server-side templates remain for initial HTML render; all JS runs client-side. New backend routes add KPI stats, DB status, and async backup job APIs.

**Tech Stack:** Rust (warp, tokio, rusqlite, rust-embed, minijinja), Vite 6, Tailwind CSS 4, ECharts 5, flatpickr 4, @tanstack/virtual-core 3.

---

## File Map

**Created:**
- `frontend/package.json`
- `frontend/vite.config.js`
- `frontend/tailwind.config.js`
- `frontend/pages/index.html`
- `frontend/pages/search.html`
- `frontend/pages/details.html`
- `frontend/pages/db.html`
- `frontend/src/theme.js`
- `frontend/src/charts.js`
- `frontend/src/main.js`
- `frontend/src/search.js`
- `frontend/src/details.js`
- `frontend/src/db.js`

**Modified:**
- `src/database.rs` — add `Stats`, `DbStatus`, `ImportRecord`, `select_stats()`, `select_db_status()`
- `src/types.rs` — add `BackupRequest`, `BackupJobResponse`, `BackupPollResponse`
- `src/web.rs` — change embed folder, add stats to index context, add 4 new routes, add backup job state to `Server`
- `src/backup.rs` — extract core logic into `backup_with_collector()` accepting a generic log sink
- `src/progress.rs` — add `LogCollector` that appends to `Arc<Mutex<Vec<String>>>`
- `.gitignore` — add `frontend/dist/`, `frontend/node_modules/`
- `.github/workflows/release.yml` — add `npm run build` step

**Deleted:**
- `static/` (entire directory, after all pages ported)

---

## Task 1: Vite + Tailwind project scaffold

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/vite.config.js`
- Create: `frontend/tailwind.config.js`
- Create: `frontend/pages/index.html` (placeholder)

- [ ] **Step 1: Create `frontend/package.json`**

```json
{
  "name": "1history-frontend",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4",
    "tailwindcss": "^4",
    "vite": "^6"
  },
  "dependencies": {
    "@tanstack/virtual-core": "^3",
    "echarts": "^5",
    "flatpickr": "^4"
  }
}
```

- [ ] **Step 2: Create `frontend/vite.config.js`**

```js
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [tailwindcss()],
  root: 'pages',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index:   resolve(__dirname, 'pages/index.html'),
        search:  resolve(__dirname, 'pages/search.html'),
        details: resolve(__dirname, 'pages/details.html'),
        db:      resolve(__dirname, 'pages/db.html'),
      },
    },
  },
  server: {
    proxy: {
      '/api':     'http://localhost:9960',
      '/details': 'http://localhost:9960',
      '/search':  'http://localhost:9960',
      '/db':      'http://localhost:9960',
    },
  },
})
```

- [ ] **Step 3: Create `frontend/tailwind.config.js`**

```js
export default {
  content: ['./pages/**/*.html', './src/**/*.js'],
}
```

- [ ] **Step 4: Create minimal `frontend/pages/index.html` to verify build works**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>1History</title>
</head>
<body>
  <h1 class="text-2xl font-bold">1History</h1>
  <script type="module" src="../src/main.js"></script>
</body>
</html>
```

- [ ] **Step 5: Create minimal `frontend/src/main.js`**

```js
console.log('1history frontend loaded')
```

- [ ] **Step 6: Install dependencies and verify build**

```bash
cd frontend && npm install && npm run build
```

Expected: `frontend/dist/` created with `index.html` and JS assets. No errors.

- [ ] **Step 7: Commit**

```bash
git add frontend/
git commit -m "feat: scaffold Vite + Tailwind frontend project"
```

---

## Task 2: Theme system (`theme.js`)

**Files:**
- Create: `frontend/src/theme.js`

- [ ] **Step 1: Create `frontend/src/theme.js`**

```js
const STORAGE_KEY = 'oh-theme'

function getPreferred() {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function initTheme() {
  const theme = getPreferred()
  document.documentElement.setAttribute('data-theme', theme)
  return theme
}

export function toggleTheme() {
  const current = document.documentElement.getAttribute('data-theme')
  const next = current === 'dark' ? 'light' : 'dark'
  document.documentElement.setAttribute('data-theme', next)
  localStorage.setItem(STORAGE_KEY, next)
  return next
}
```

- [ ] **Step 2: Create `frontend/src/base.css` with CSS variables**

```css
@import "tailwindcss";

:root {
  --bg: #ffffff;
  --bg-card: #f8f9fa;
  --text: #1a1a1a;
  --text-muted: #6b7280;
  --border: #e5e7eb;
  --accent: #4A90D9;
  --accent-hover: #357abd;
}

[data-theme="dark"] {
  --bg: #0f1117;
  --bg-card: #1a1d2e;
  --text: #e2e8f0;
  --text-muted: #6b7280;
  --border: #2a2d3e;
  --accent: #6366f1;
  --accent-hover: #4f46e5;
}

body {
  background-color: var(--bg);
  color: var(--text);
  font-family: system-ui, -apple-system, sans-serif;
}

.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 16px;
}

.btn {
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 6px 14px;
  cursor: pointer;
  font-size: 14px;
}
.btn:hover { background: var(--accent-hover); }

.btn-outline {
  background: transparent;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 14px;
  cursor: pointer;
  font-size: 14px;
}
.btn-outline:hover { border-color: var(--accent); color: var(--accent); }

nav {
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  padding: 0 24px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-brand {
  font-weight: 700;
  font-size: 18px;
  color: var(--text);
  text-decoration: none;
}

table { width: 100%; border-collapse: collapse; }
th { text-align: left; padding: 8px 12px; font-size: 12px; color: var(--text-muted); text-transform: uppercase; border-bottom: 1px solid var(--border); }
td { padding: 8px 12px; font-size: 13px; border-bottom: 1px solid var(--border); }
tr:last-child td { border-bottom: none; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/theme.js frontend/src/base.css
git commit -m "feat: add theme system with dark/light CSS variables"
```

---

## Task 3: ECharts wrapper (`charts.js`)

**Files:**
- Create: `frontend/src/charts.js`

- [ ] **Step 1: Create `frontend/src/charts.js`**

```js
import * as echarts from 'echarts/core'
import { LineChart, PieChart } from 'echarts/charts'
import {
  TitleComponent, TooltipComponent, LegendComponent,
  GridComponent, DataZoomComponent, ToolboxComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([
  LineChart, PieChart,
  TitleComponent, TooltipComponent, LegendComponent,
  GridComponent, DataZoomComponent, ToolboxComponent,
  CanvasRenderer,
])

export function initDailyChart(el, data, onClickDate) {
  const chart = echarts.init(el)
  chart.setOption({
    color: ['var(--accent)'],
    title: { text: 'Daily PV', subtext: 'Click any point to view details' },
    tooltip: {
      trigger: 'axis',
      formatter: (params) => {
        const d = new Date(params[0].value[0])
        return `${d.toLocaleDateString()}<br/>PV: ${params[0].value[1]}`
      },
    },
    toolbox: { feature: { saveAsImage: { show: true }, dataView: { show: true, readOnly: false } } },
    dataZoom: [{ type: 'inside' }, { type: 'slider' }],
    xAxis: { type: 'time' },
    yAxis: { name: 'PV', type: 'value' },
    series: [{
      name: 'Page View',
      type: 'line',
      showAllSymbol: true,
      data: data.map(([ts, cnt]) => [new Date(ts), cnt]),
    }],
  })
  chart.on('click', (params) => {
    const d = new Date(params.value[0])
    const ymd = d.toISOString().slice(0, 10)
    onClickDate(ymd)
  })
  return chart
}

export function initPieChart(el, data, title, onClickItem) {
  const chart = echarts.init(el)
  const top10 = data.slice(0, 10).map(([name, value]) => ({
    name: name.length > 50 ? name.slice(0, 50) : name,
    value,
  }))
  chart.setOption({
    title: { text: title, left: 'center' },
    tooltip: { trigger: 'item', formatter: '{a}<br/>{b}: {c} ({d}%)' },
    legend: { orient: 'vertical', left: 'left', data: top10.map(d => d.name) },
    toolbox: { feature: { saveAsImage: { show: true } } },
    series: [{
      name: title,
      type: 'pie',
      radius: '65%',
      center: ['50%', '60%'],
      data: top10,
    }],
  })
  if (onClickItem) {
    chart.on('click', (params) => onClickItem(params.name))
  }
  return chart
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/charts.js
git commit -m "feat: add ECharts 5 wrappers for daily PV and pie charts"
```

---

## Task 4: Backend — `select_stats` and `select_db_status`

**Files:**
- Modify: `src/database.rs`
- Modify: `src/types.rs`

- [ ] **Step 1: Add `Stats` and `DbStatus` structs to `src/types.rs`**

Add after the existing `VisitDetail` struct:

```rust
#[derive(Debug, Serialize)]
pub struct Stats {
    pub total_visits: i64,
    pub unique_domains: i64,
    pub active_days: i64,
    pub today_visits: i64,
}

#[derive(Debug, Serialize)]
pub struct ImportRecord {
    pub data_path: String,
    pub last_import: i64,  // unix ms
}

#[derive(Debug, Serialize)]
pub struct DbStatus {
    pub file_path: String,
    pub file_size_bytes: u64,
    pub total_visits: i64,
    pub min_time: i64,  // unix ms
    pub max_time: i64,  // unix ms
    pub chrome_visits: i64,
    pub firefox_visits: i64,
    pub safari_visits: i64,
    pub import_records: Vec<ImportRecord>,
}
```

- [ ] **Step 2: Add `select_stats` to `src/database.rs`**

Add after the `select_min_max_time` method:

```rust
pub fn select_stats(&self, start: i64, end: i64) -> Result<crate::types::Stats> {
    let today_start = crate::util::tomorrow_midnight() - 86_400_000;
    let today_end = crate::util::tomorrow_midnight() - 1;
    let sql = r#"
SELECT
    (SELECT count(1) FROM onehistory_visits WHERE visit_time BETWEEN :start AND :end) AS total_visits,
    (SELECT count(DISTINCT u.id) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE v.visit_time BETWEEN :start AND :end) AS unique_domains,
    (SELECT count(DISTINCT strftime('%Y-%m-%d', visit_time/1000000, 'unixepoch', 'localtime')) FROM onehistory_visits WHERE visit_time BETWEEN :start AND :end) AS active_days,
    (SELECT count(1) FROM onehistory_visits WHERE visit_time BETWEEN :today_start AND :today_end) AS today_visits
"#;
    let conn = self.conn.lock().unwrap();
    let mut stat = conn.prepare(sql)?;
    let result = stat.query_row(
        rusqlite::named_params! {
            ":start": Self::unixepoch_to_prtime(start),
            ":end": Self::unixepoch_to_prtime(end),
            ":today_start": Self::unixepoch_to_prtime(today_start),
            ":today_end": Self::unixepoch_to_prtime(today_end),
        },
        |row| Ok(crate::types::Stats {
            total_visits: row.get(0)?,
            unique_domains: row.get(1)?,
            active_days: row.get(2)?,
            today_visits: row.get(3)?,
        }),
    )?;
    Ok(result)
}
```

- [ ] **Step 3: Add `select_db_status` to `src/database.rs`**

Add the method and the file path field to the `Database` struct:

```rust
// In the Database struct, add file_path field:
pub(crate) struct Database {
    conn: Mutex<Connection>,
    persist_batch: usize,
    pub file_path: String,
}

// In Database::open, store file path:
pub fn open(sqlite_datafile: String) -> Result<Database> {
    let conn = Connection::open(&sqlite_datafile)?;
    let db = Self {
        conn: Mutex::new(conn),
        persist_batch: DEFAULT_BATCH_NUM,
        file_path: sqlite_datafile,
    };
    db.init().context("init")?;
    Ok(db)
}

// New method:
pub fn select_db_status(&self) -> Result<crate::types::DbStatus> {
    use std::fs;
    let file_size_bytes = fs::metadata(&self.file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let conn = self.conn.lock().unwrap();

    let (total_visits, min_time, max_time): (i64, i64, i64) = conn.query_row(
        "SELECT count(1), CAST(min(visit_time)/1000 AS integer), CAST(max(visit_time)/1000 AS integer) FROM onehistory_visits",
        [],
        |row| Ok((row.get(0)?, row.get(1).unwrap_or(0), row.get(2).unwrap_or(0))),
    )?;

    let browser_sql = r#"
SELECT
    sum(CASE WHEN u.url LIKE 'chrome%' OR u.url NOT LIKE 'place%' AND u.url NOT LIKE 'safari%' THEN 1 ELSE 0 END),
    sum(CASE WHEN u.url LIKE 'place%' THEN 1 ELSE 0 END),
    sum(CASE WHEN u.url LIKE 'safari%' THEN 1 ELSE 0 END)
FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id
"#;
    // Per-browser counts via import_records path name heuristic (more reliable)
    let chrome_visits: i64 = conn.query_row(
        "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url NOT LIKE 'place%' AND u.url NOT LIKE 'safari%' AND u.url NOT LIKE 'about:%'",
        [], |row| row.get(0)).unwrap_or(0);
    let firefox_visits: i64 = conn.query_row(
        "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url LIKE 'place%'",
        [], |row| row.get(0)).unwrap_or(0);
    let safari_visits: i64 = conn.query_row(
        "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url LIKE 'safari%'",
        [], |row| row.get(0)).unwrap_or(0);
    let _ = browser_sql; // suppress unused warning

    let mut import_stat = conn.prepare(
        "SELECT data_path, CAST(last_import/1000 AS integer) FROM import_records ORDER BY last_import DESC"
    )?;
    let import_records: Vec<crate::types::ImportRecord> = import_stat
        .query_map([], |row| Ok(crate::types::ImportRecord {
            data_path: row.get(0)?,
            last_import: row.get(1)?,
        }))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(crate::types::DbStatus {
        file_path: self.file_path.clone(),
        file_size_bytes,
        total_visits,
        min_time,
        max_time,
        chrome_visits,
        firefox_visits,
        safari_visits,
        import_records,
    })
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: `Finished` with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/database.rs src/types.rs
git commit -m "feat: add select_stats and select_db_status to Database"
```

---

## Task 5: Backend — backup async API + new routes

**Files:**
- Modify: `src/backup.rs`
- Modify: `src/progress.rs`
- Modify: `src/types.rs`
- Modify: `src/web.rs`

- [ ] **Step 1: Add `LogCollector` to `src/progress.rs`**

```rust
use std::sync::{Arc, Mutex};

pub struct LogCollector {
    lines: Arc<Mutex<Vec<String>>>,
    label: String,
    total: u64,
    done: Arc<Mutex<u64>>,
}

impl LogCollector {
    pub fn new(label: String, total: u64, lines: Arc<Mutex<Vec<String>>>) -> Self {
        Self { lines, label, total, done: Arc::new(Mutex::new(0)) }
    }
}

impl ProgressCollector for LogCollector {
    fn inc(&self, delta: u64) {
        let mut d = self.done.lock().unwrap();
        *d += delta;
        if *d % 500 == 0 || *d == self.total {
            self.lines.lock().unwrap().push(
                format!("[{}] {}/{}", self.label, d, self.total)
            );
        }
    }
    fn finish(&self) {
        let d = *self.done.lock().unwrap();
        self.lines.lock().unwrap().push(
            format!("[{}] done ({} records)", self.label, d)
        );
    }
}
```

- [ ] **Step 2: Add a `backup_to_log` function in `src/backup.rs`**

This is the existing `backup()` logic refactored to use `LogCollector` and return structured results:

```rust
use crate::progress::LogCollector;
use std::sync::{Arc, Mutex};

pub struct BackupResult {
    pub found: usize,
    pub imported: usize,
    pub duplicated: usize,
}

pub fn backup_to_log(
    history_files: Vec<String>,
    db_file: String,
    dry_run: bool,
    log_lines: Arc<Mutex<Vec<String>>>,
) -> anyhow::Result<BackupResult> {
    let (start, end) = crate::util::full_timerange();
    let db = crate::database::Database::open(db_file).context("open 1History DB")?;

    let mut found = 0;
    let mut total_affected = 0;
    let mut total_duplicated = 0;

    let mut persist = |history_file: &str| {
        let s = crate::source::Source::open(history_file).context("open")?;
        let rows = s.select(start, end).context("select")?.collect::<Vec<_>>();
        found += rows.len();
        log_lines.lock().unwrap().push(format!("Processing {} ({} records)...", history_file, rows.len()));
        let collector = LogCollector::new(history_file.to_string(), rows.len() as u64, Arc::clone(&log_lines));
        if !dry_run {
            let (affected, duplicated) = db.persist(s.path(), rows, collector).context("persist")?;
            total_affected += affected;
            total_duplicated += duplicated;
        }
        log_lines.lock().unwrap().push(format!("Done: {}", history_file));
        Ok::<_, anyhow::Error>(())
    };

    for his_file in &history_files {
        if let Err(e) = persist(his_file) {
            let msg = format!("{e:?}");
            if msg.contains("The database file is locked") {
                let mut tmp = match tempfile::NamedTempFile::new() { Ok(f) => f, Err(_) => continue };
                let body = match std::fs::read(his_file) { Ok(v) => v, Err(_) => continue };
                if tmp.write_all(&body).is_err() { continue; }
                let path = tmp.into_temp_path();
                let path = path.to_string_lossy().to_string();
                let _ = persist(&path);
            }
        }
    }

    Ok(BackupResult { found, imported: total_affected, duplicated: total_duplicated })
}
```

- [ ] **Step 3: Add backup API types to `src/types.rs`**

```rust
#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub files: Vec<String>,
    pub disable_detect: bool,
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct BackupJobResponse {
    pub job_id: String,
}

#[derive(Debug, Serialize)]
pub struct BackupPollResponse {
    pub status: String,  // "running" | "done" | "error"
    pub log_lines: Vec<String>,
    pub summary: Option<BackupSummary>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BackupSummary {
    pub found: usize,
    pub imported: usize,
    pub duplicated: usize,
    pub error: Option<String>,
}
```

- [ ] **Step 4: Add backup job state and new routes to `src/web.rs`**

Add the job store type alias and update the `Server` struct:

```rust
// Add near top of web.rs, after existing use statements:
use std::collections::HashMap;
use uuid::Uuid;
use crate::types::{BackupRequest, BackupJobResponse, BackupPollResponse, BackupSummary};
use crate::backup::backup_to_log;
use crate::util::detect_history_files;

type JobId = String;

#[derive(Clone)]
struct BackupJob {
    log_lines: Arc<std::sync::Mutex<Vec<String>>>,
    status: Arc<std::sync::Mutex<String>>,
    summary: Arc<std::sync::Mutex<Option<BackupSummary>>>,
}

type JobStore = Arc<std::sync::Mutex<HashMap<JobId, BackupJob>>>;

// Update Server struct:
struct Server {
    db: Arc<Database>,
    addr: SocketAddr,
    db_file: String,
    jobs: JobStore,
}

// Update try_new:
fn try_new(addr: String, db_filepath: String) -> Result<Self> {
    Ok(Self {
        db: Arc::new(Database::open(db_filepath.clone()).context("open db")?),
        addr: addr.parse()?,
        db_file: db_filepath,
        jobs: Arc::new(std::sync::Mutex::new(HashMap::new())),
    })
}
```

- [ ] **Step 5: Add `uuid` to `Cargo.toml`**

```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 6: Add the four new handler methods to `Server` impl in `src/web.rs`**

```rust
fn with_jobs(jobs: JobStore) -> impl Filter<Extract = (JobStore,), Error = Infallible> + Clone {
    warp::any().map(move || jobs.clone())
}

fn with_db_file(db_file: String) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || db_file.clone())
}

async fn db_status(db: Arc<Database>) -> Result<impl Reply, Rejection> {
    let status = db.select_db_status().map_err(ServerError::from)?;
    Ok(warp::reply::json(&status))
}

async fn start_backup(
    db_file: String,
    jobs: JobStore,
    req: BackupRequest,
) -> Result<impl Reply, Rejection> {
    let job_id = Uuid::new_v4().to_string();
    let log_lines: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let status = Arc::new(std::sync::Mutex::new("running".to_string()));
    let summary_store: Arc<std::sync::Mutex<Option<BackupSummary>>> = Arc::new(std::sync::Mutex::new(None));

    let job = BackupJob {
        log_lines: Arc::clone(&log_lines),
        status: Arc::clone(&status),
        summary: Arc::clone(&summary_store),
    };
    jobs.lock().unwrap().insert(job_id.clone(), job);

    let db_file_clone = db_file.clone();
    tokio::task::spawn_blocking(move || {
        let mut files = if req.disable_detect { vec![] } else { detect_history_files() };
        files.extend(req.files);
        match backup_to_log(files, db_file_clone, req.dry_run, Arc::clone(&log_lines)) {
            Ok(result) => {
                *summary_store.lock().unwrap() = Some(BackupSummary {
                    found: result.found,
                    imported: result.imported,
                    duplicated: result.duplicated,
                    error: None,
                });
                *status.lock().unwrap() = "done".to_string();
            }
            Err(e) => {
                *summary_store.lock().unwrap() = Some(BackupSummary {
                    found: 0, imported: 0, duplicated: 0,
                    error: Some(format!("{e:?}")),
                });
                *status.lock().unwrap() = "error".to_string();
            }
        }
    });

    Ok(warp::reply::json(&BackupJobResponse { job_id }))
}

async fn poll_backup(job_id: String, jobs: JobStore) -> Result<impl Reply, Rejection> {
    let jobs = jobs.lock().unwrap();
    let job = jobs.get(&job_id).ok_or_else(|| {
        warp::reject::custom(ClientError { e: "job not found".to_string() })
    })?;
    let status = job.status.lock().unwrap().clone();
    let log_lines = job.log_lines.lock().unwrap().clone();
    let summary = job.summary.lock().unwrap().clone();
    Ok(warp::reply::json(&BackupPollResponse { status, log_lines, summary }))
}
```

- [ ] **Step 7: Register the new routes in `Server::serve`**

```rust
// In serve(), add these routes and wire them in:
let db_status_route = warp::path!("api" / "db" / "status")
    .and(Self::with_db(self.db.clone()))
    .and_then(Self::db_status);

let backup_start = warp::path!("api" / "backup")
    .and(warp::post())
    .and(Self::with_db_file(self.db_file.clone()))
    .and(Self::with_jobs(self.jobs.clone()))
    .and(warp::body::json())
    .and_then(Self::start_backup);

let backup_poll = warp::path!("api" / "backup" / String)
    .and(warp::get())
    .and(Self::with_jobs(self.jobs.clone()))
    .and_then(Self::poll_backup);

let db_page = warp::path("db")
    .and(warp::path::end())
    .map(|| {
        let asset = Asset::get("db.html").unwrap();
        warp::reply::html(String::from_utf8_lossy(&asset.data).to_string())
    });

// Update routes chain to include new routes:
let routes = detail
    .or(search)
    .or(db_status_route)
    .or(backup_start)
    .or(backup_poll)
    .or(db_page)
    .or(index)
    .or(static_route)
    .recover(Self::handle_rejection);
```

- [ ] **Step 8: Also pass `stats` to index handler**

In the `index` handler, after `domain_top100`, add:

```rust
let stats = db
    .select_stats(start, end)
    .context("stats")
    .map_err(ServerError::from)?;
```

And in the template context:
```rust
stats => stats,
```

- [ ] **Step 9: Verify build**

```bash
cargo build 2>&1
```

Expected: `Finished` with no errors.

- [ ] **Step 10: Commit**

```bash
git add src/backup.rs src/progress.rs src/types.rs src/web.rs Cargo.toml Cargo.lock
git commit -m "feat: add async backup API and DB status endpoint"
```

---

## Task 6: Index page HTML + JS

**Files:**
- Modify: `frontend/pages/index.html`
- Modify: `frontend/src/main.js`

The index page is server-rendered by Minijinja, so the HTML template uses `{{ }}` syntax. Vite treats these files as static — the Minijinja variables are injected at request time by the Rust server.

- [ ] **Step 1: Write `frontend/pages/index.html`**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="icon" href="/static/img/history.ico">
  <title>1History</title>
  <link rel="stylesheet" href="../src/base.css">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/flatpickr/dist/flatpickr.min.css">
</head>
<body>
  <nav>
    <a class="nav-brand" href="/">1History</a>
    <div style="display:flex;align-items:center;gap:10px">
      <input id="date-range" class="card" style="width:240px;cursor:pointer;font-size:13px" readonly placeholder="Date range">
      <input id="keyword" type="search" placeholder="Search..." style="padding:6px 10px;border:1px solid var(--border);border-radius:6px;background:var(--bg);color:var(--text);font-size:13px;width:220px">
      <button class="btn" id="submit">Search</button>
      <a id="view-list" class="btn-outline" target="_blank" style="text-decoration:none">
        List <span id="view-list-count"></span>
      </a>
      <a href="/db" class="btn-outline" style="text-decoration:none">DB</a>
      <button id="theme-toggle" class="btn-outline" title="Toggle theme">🌙</button>
    </div>
  </nav>

  <div style="padding:24px;max-width:1400px;margin:0 auto">
    <!-- KPI cards -->
    <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:24px">
      <div class="card" style="text-align:center">
        <div style="font-size:28px;font-weight:700;color:var(--accent)" id="kpi-total">—</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Total Visits</div>
      </div>
      <div class="card" style="text-align:center">
        <div style="font-size:28px;font-weight:700;color:var(--accent)" id="kpi-domains">—</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Unique Domains</div>
      </div>
      <div class="card" style="text-align:center">
        <div style="font-size:28px;font-weight:700;color:var(--accent)" id="kpi-days">—</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Active Days</div>
      </div>
      <div class="card" style="text-align:center">
        <div style="font-size:28px;font-weight:700;color:var(--accent)" id="kpi-today">—</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:4px">Today</div>
      </div>
    </div>

    <!-- Daily PV chart -->
    <div class="card" style="margin-bottom:16px">
      <div id="daily-chart" style="height:360px"></div>
    </div>

    <!-- Pie charts -->
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:16px">
      <div class="card"><div id="title-pie" style="height:320px"></div></div>
      <div class="card"><div id="domain-pie" style="height:320px"></div></div>
    </div>

    <!-- Tables -->
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">
      <div class="card">
        <h3 style="font-size:14px;font-weight:600;margin-bottom:12px">TOP 100 by title</h3>
        <div style="overflow-x:auto">
          <table>
            <thead><tr><th>Count</th><th>Title</th></tr></thead>
            <tbody id="title-table"></tbody>
          </table>
        </div>
      </div>
      <div class="card">
        <h3 style="font-size:14px;font-weight:600;margin-bottom:12px">TOP 100 by domain</h3>
        <div style="overflow-x:auto">
          <table>
            <thead><tr><th>Count</th><th>Domain</th></tr></thead>
            <tbody id="domain-table"></tbody>
          </table>
        </div>
      </div>
    </div>
  </div>

  <script type="module" src="../src/main.js"></script>
  <!-- Server-injected data -->
  <script id="server-data" type="application/json">
  {
    "start": {{ start }},
    "end": {{ end }},
    "minTime": {{ min_time }},
    "maxTime": {{ max_time }},
    "dailyCounts": {{ daily_counts }},
    "titleTop100": {{ title_top100 }},
    "domainTop100": {{ domain_top100 }},
    "visitCount": {{ visit_details | length }},
    "keyword": "{{ keyword | escape }}",
    "startYmd": "{{ start_ymd }}",
    "endYmd": "{{ end_ymd }}",
    "stats": {{ stats }}
  }
  </script>
</body>
</html>
```

- [ ] **Step 2: Write `frontend/src/main.js`**

```js
import { initTheme, toggleTheme } from './theme.js'
import { initDailyChart, initPieChart } from './charts.js'
import flatpickr from 'flatpickr'
import '../src/base.css'

const d = JSON.parse(document.getElementById('server-data').textContent)

// Theme
const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

// KPI cards
document.getElementById('kpi-total').textContent = d.stats.total_visits.toLocaleString()
document.getElementById('kpi-domains').textContent = d.stats.unique_domains.toLocaleString()
document.getElementById('kpi-days').textContent = d.stats.active_days.toLocaleString()
document.getElementById('kpi-today').textContent = d.stats.today_visits.toLocaleString()

// Date range picker
const fp = flatpickr('#date-range', {
  mode: 'range',
  dateFormat: 'Y-m-d',
  defaultDate: [d.startYmd, d.endYmd],
  minDate: new Date(d.minTime),
  maxDate: new Date(d.maxTime),
})
document.getElementById('date-range')._flatpickr = fp

// Keyword pre-fill
document.getElementById('keyword').value = d.keyword

// Search handler
function doSearch() {
  const kw = document.getElementById('keyword').value
  const dates = fp.selectedDates
  if (dates.length < 2) return
  const fmt = (dt) => dt.toISOString().slice(0, 10)
  window.location = `/?start=${fmt(dates[0])}&end=${fmt(dates[1])}&keyword=${encodeURIComponent(kw)}`
}
document.getElementById('submit').addEventListener('click', doSearch)
document.getElementById('keyword').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') doSearch()
})

// List button
const listBtn = document.getElementById('view-list')
const dates = fp.selectedDates
const fmt = (dt) => dt.toISOString().slice(0, 10)
const listParams = `start=${d.startYmd}&end=${d.endYmd}${d.keyword ? '&keyword=' + encodeURIComponent(d.keyword) : ''}`
listBtn.href = `/search?${listParams}`
document.getElementById('view-list-count').textContent = d.visitCount.toLocaleString()

// Charts
const keyword = d.keyword
initDailyChart(
  document.getElementById('daily-chart'),
  d.dailyCounts,
  (ymd) => window.open(`/details/${ymd}?keyword=${encodeURIComponent(keyword)}`, '_blank')
)
initPieChart(document.getElementById('title-pie'), d.titleTop100, 'TOP 10 by Title', null)
initPieChart(
  document.getElementById('domain-pie'),
  d.domainTop100,
  'TOP 10 by Domain',
  (domain) => {
    document.getElementById('keyword').value = domain
    doSearch()
  }
)

// Tables
function fillTable(tbodyId, rows) {
  const tbody = document.getElementById(tbodyId)
  tbody.innerHTML = rows.map(([name, cnt]) =>
    `<tr><td>${cnt}</td><td>${name}</td></tr>`
  ).join('')
}
fillTable('title-table', d.titleTop100)
fillTable('domain-table', d.domainTop100)
```

- [ ] **Step 3: Build frontend and verify**

```bash
cd frontend && npm run build
```

Expected: no errors, `dist/` contains `index.html`.

- [ ] **Step 4: Commit**

```bash
git add frontend/pages/index.html frontend/src/main.js
git commit -m "feat: build index page with KPI cards, charts, and theme toggle"
```

---

## Task 7: Search page

**Files:**
- Create: `frontend/pages/search.html`
- Create: `frontend/src/search.js`

- [ ] **Step 1: Write `frontend/pages/search.html`**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="icon" href="/static/img/history.ico">
  <title>1History Search</title>
  <link rel="stylesheet" href="../src/base.css">
</head>
<body>
  <nav>
    <div style="display:flex;align-items:center;gap:16px">
      <a class="nav-brand" href="/">1History</a>
      <span style="color:var(--text-muted);font-size:13px" id="filter-summary"></span>
    </div>
    <button id="theme-toggle" class="btn-outline" title="Toggle theme">🌙</button>
  </nav>

  <div style="padding:24px;max-width:1200px;margin:0 auto">
    <div style="display:flex;align-items:center;gap:12px;margin-bottom:16px">
      <input id="filter-input" type="search" placeholder="Filter results..." style="flex:1;padding:8px 12px;border:1px solid var(--border);border-radius:6px;background:var(--bg);color:var(--text);font-size:14px">
      <span style="color:var(--text-muted);font-size:13px;white-space:nowrap" id="result-count"></span>
    </div>
    <div class="card" style="padding:0;overflow:hidden">
      <div style="display:grid;grid-template-columns:160px 1fr;padding:8px 16px;background:var(--bg-card);border-bottom:1px solid var(--border)">
        <span style="font-size:12px;color:var(--text-muted);text-transform:uppercase;font-weight:600">Time</span>
        <span style="font-size:12px;color:var(--text-muted);text-transform:uppercase;font-weight:600">Title</span>
      </div>
      <div id="virtual-container" style="height:calc(100vh - 180px);overflow-y:auto;position:relative">
        <div id="virtual-total-height"></div>
        <div id="virtual-rows" style="position:absolute;top:0;left:0;right:0"></div>
      </div>
    </div>
  </div>

  <script type="application/json" id="server-data">
  {
    "visits": {{ visit_details }},
    "keyword": "{{ keyword | escape }}",
    "startYmd": "{{ start_ymd }}",
    "endYmd": "{{ end_ymd }}"
  }
  </script>
  <script type="module" src="../src/search.js"></script>
</body>
</html>
```

- [ ] **Step 2: Write `frontend/src/search.js`**

```js
import { initTheme, toggleTheme } from './theme.js'
import { VirtualItem, Virtualizer } from '@tanstack/virtual-core'
import '../src/base.css'

void VirtualItem // type import

const d = JSON.parse(document.getElementById('server-data').textContent)

// Theme
const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

// Filter summary
const summaryEl = document.getElementById('filter-summary')
summaryEl.textContent = `${d.startYmd} – ${d.endYmd}${d.keyword ? ' · "' + d.keyword + '"' : ''}`

// Data + filter
let allVisits = d.visits
let filtered = allVisits

const filterInput = document.getElementById('filter-input')
const countEl = document.getElementById('result-count')

function updateCount() {
  countEl.textContent = `${filtered.length.toLocaleString()} records`
}
updateCount()

filterInput.addEventListener('input', () => {
  const q = filterInput.value.toLowerCase()
  filtered = q
    ? allVisits.filter(v => v.url.toLowerCase().includes(q) || v.title.toLowerCase().includes(q))
    : allVisits
  updateCount()
  virtualizer.setOptions({ ...virtualizer.options, count: filtered.length })
  virtualizer.measure()
  renderRows()
})

// Virtual scroll
const ROW_HEIGHT = 36
const container = document.getElementById('virtual-container')
const totalHeightEl = document.getElementById('virtual-total-height')
const rowsEl = document.getElementById('virtual-rows')

const virtualizer = new Virtualizer({
  count: filtered.length,
  getScrollElement: () => container,
  estimateSize: () => ROW_HEIGHT,
  overscan: 10,
  scrollMargin: 0,
  onChange: () => renderRows(),
})

function formatDatetime(ms) {
  return new Date(ms * 1000).toLocaleString(undefined, {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  })
}

function renderRows() {
  const items = virtualizer.getVirtualItems()
  totalHeightEl.style.height = virtualizer.getTotalSize() + 'px'
  rowsEl.style.transform = items.length ? `translateY(${items[0].start}px)` : 'translateY(0)'
  rowsEl.innerHTML = items.map(item => {
    const v = filtered[item.index]
    const title = v.title || v.url
    return `<div style="display:grid;grid-template-columns:160px 1fr;padding:8px 16px;border-bottom:1px solid var(--border);height:${ROW_HEIGHT}px;align-items:center">
      <span style="font-size:12px;color:var(--text-muted)">${formatDatetime(v.visit_time)}</span>
      <a href="${v.url}" style="font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="${v.title}">${title}</a>
    </div>`
  }).join('')
}

virtualizer._willUpdate()
renderRows()

container.addEventListener('scroll', () => {
  virtualizer._willUpdate()
  renderRows()
})
```

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/pages/search.html frontend/src/search.js
git commit -m "feat: build search page with virtual scroll and client-side filter"
```

---

## Task 8: Details page

**Files:**
- Create: `frontend/pages/details.html`
- Create: `frontend/src/details.js`

- [ ] **Step 1: Write `frontend/pages/details.html`**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="icon" href="/static/img/history.ico">
  <title>1History — {{ ymd }}</title>
  <link rel="stylesheet" href="../src/base.css">
</head>
<body>
  <nav>
    <div style="display:flex;align-items:center;gap:16px">
      <a class="nav-brand" href="/">1History</a>
      <a href="/details/{{ format_as_ymd(ymd_ts - 3600000*24) }}?{{ {"keyword": keyword} | urlencode }}" class="btn-outline" style="text-decoration:none;font-size:12px">← Yesterday</a>
      <span style="font-weight:600">{{ ymd }}</span>
      <a href="/details/{{ format_as_ymd(ymd_ts + 3600000*24) }}?{{ {"keyword": keyword} | urlencode }}" class="btn-outline" style="text-decoration:none;font-size:12px">Tomorrow →</a>
    </div>
    <div style="display:flex;align-items:center;gap:10px">
      <input id="keyword" type="search" placeholder="Filter..." style="padding:6px 10px;border:1px solid var(--border);border-radius:6px;background:var(--bg);color:var(--text);font-size:13px" value="{{ keyword }}">
      <button class="btn" id="submit">Search</button>
      <span style="color:var(--text-muted);font-size:13px">{{ visit_details | length }} results</span>
      <button id="theme-toggle" class="btn-outline" title="Toggle theme">🌙</button>
    </div>
  </nav>

  <div style="padding:24px;max-width:1200px;margin:0 auto">
    <div class="card" style="padding:0;overflow:hidden">
      <table>
        <thead><tr><th style="width:120px">Time</th><th>Title</th></tr></thead>
        <tbody>
          {% for detail in visit_details %}
          <tr>
            <td style="white-space:nowrap;color:var(--text-muted)">{{ format_as_hms(detail.visit_time) }}</td>
            <td><a href="{{ detail.url }}">{{ format_title(detail.title, detail.url) }}</a></td>
          </tr>
          {% endfor %}
        </tbody>
      </table>
    </div>
  </div>

  <script type="module" src="../src/details.js"></script>
</body>
</html>
```

- [ ] **Step 2: Write `frontend/src/details.js`**

```js
import { initTheme, toggleTheme } from './theme.js'
import '../src/base.css'

const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

document.getElementById('submit').addEventListener('click', () => {
  const kw = document.getElementById('keyword').value
  const ymd = window.location.pathname.split('/').pop()
  window.location = `/details/${ymd}?keyword=${encodeURIComponent(kw)}`
})
document.getElementById('keyword').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') document.getElementById('submit').click()
})
```

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/pages/details.html frontend/src/details.js
git commit -m "feat: build details page with new theme"
```

---

## Task 9: DB management page

**Files:**
- Create: `frontend/pages/db.html`
- Create: `frontend/src/db.js`

- [ ] **Step 1: Write `frontend/pages/db.html`**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="icon" href="/static/img/history.ico">
  <title>1History — Database</title>
  <link rel="stylesheet" href="../src/base.css">
</head>
<body>
  <nav>
    <div style="display:flex;align-items:center;gap:16px">
      <a class="nav-brand" href="/">1History</a>
      <span style="color:var(--text-muted);font-size:13px">Database Management</span>
    </div>
    <button id="theme-toggle" class="btn-outline" title="Toggle theme">🌙</button>
  </nav>

  <div style="padding:24px;max-width:1000px;margin:0 auto;display:flex;flex-direction:column;gap:20px">

    <!-- Status section -->
    <div class="card">
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px">
        <h2 style="font-size:16px;font-weight:700">Database Status</h2>
        <button class="btn-outline" id="refresh-status" style="font-size:12px">Refresh</button>
      </div>
      <div id="status-content">
        <p style="color:var(--text-muted)">Loading...</p>
      </div>
    </div>

    <!-- Backup section -->
    <div class="card">
      <h2 style="font-size:16px;font-weight:700;margin-bottom:16px">Backup</h2>

      <div style="margin-bottom:12px">
        <label style="font-size:13px;font-weight:500;display:block;margin-bottom:6px">
          Extra history files (one path per line)
        </label>
        <textarea id="extra-files" rows="3" style="width:100%;padding:8px;border:1px solid var(--border);border-radius:6px;background:var(--bg);color:var(--text);font-size:13px;resize:vertical;font-family:monospace" placeholder="/path/to/History&#10;/path/to/places.sqlite"></textarea>
      </div>

      <div style="display:flex;gap:20px;margin-bottom:16px">
        <label style="display:flex;align-items:center;gap:6px;font-size:13px;cursor:pointer">
          <input type="checkbox" id="disable-detect"> Disable auto-detect
        </label>
        <label style="display:flex;align-items:center;gap:6px;font-size:13px;cursor:pointer">
          <input type="checkbox" id="dry-run"> Dry run
        </label>
      </div>

      <button class="btn" id="start-backup">Start Backup</button>

      <div id="backup-progress" style="display:none;margin-top:16px">
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
          <span id="backup-status-badge" style="font-size:12px;padding:2px 8px;border-radius:20px;background:var(--accent);color:#fff">running</span>
          <span style="font-size:13px;color:var(--text-muted)" id="backup-status-text">Backup in progress...</span>
        </div>
        <div style="background:var(--bg);border:1px solid var(--border);border-radius:6px;padding:10px;max-height:200px;overflow-y:auto;font-family:monospace;font-size:12px" id="backup-log"></div>
        <div id="backup-summary" style="display:none;margin-top:12px;padding:12px;background:var(--bg-card);border-radius:6px;border:1px solid var(--border)"></div>
      </div>
    </div>

  </div>
  <script type="module" src="../src/db.js"></script>
</body>
</html>
```

- [ ] **Step 2: Write `frontend/src/db.js`**

```js
import { initTheme, toggleTheme } from './theme.js'
import '../src/base.css'

const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

function fmtBytes(bytes) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(2) + ' MB'
}

function fmtDate(ms) {
  if (!ms) return '—'
  return new Date(ms).toLocaleString()
}

async function loadStatus() {
  document.getElementById('status-content').innerHTML = '<p style="color:var(--text-muted)">Loading...</p>'
  try {
    const res = await fetch('/api/db/status')
    const s = await res.json()
    document.getElementById('status-content').innerHTML = `
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:16px">
        <div><span style="font-size:12px;color:var(--text-muted)">File</span><div style="font-size:13px;font-family:monospace;margin-top:2px">${s.file_path}</div></div>
        <div><span style="font-size:12px;color:var(--text-muted)">Size</span><div style="font-size:13px;margin-top:2px">${fmtBytes(s.file_size_bytes)}</div></div>
        <div><span style="font-size:12px;color:var(--text-muted)">Total visits</span><div style="font-size:20px;font-weight:700;color:var(--accent);margin-top:2px">${s.total_visits.toLocaleString()}</div></div>
        <div><span style="font-size:12px;color:var(--text-muted)">Date range</span><div style="font-size:13px;margin-top:2px">${fmtDate(s.min_time)} → ${fmtDate(s.max_time)}</div></div>
      </div>
      <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-bottom:16px">
        <div class="card" style="text-align:center;padding:10px">
          <div style="font-size:18px;font-weight:700;color:var(--accent)">${s.chrome_visits.toLocaleString()}</div>
          <div style="font-size:11px;color:var(--text-muted)">Chrome / Chromium</div>
        </div>
        <div class="card" style="text-align:center;padding:10px">
          <div style="font-size:18px;font-weight:700;color:var(--accent)">${s.firefox_visits.toLocaleString()}</div>
          <div style="font-size:11px;color:var(--text-muted)">Firefox</div>
        </div>
        <div class="card" style="text-align:center;padding:10px">
          <div style="font-size:18px;font-weight:700;color:var(--accent)">${s.safari_visits.toLocaleString()}</div>
          <div style="font-size:11px;color:var(--text-muted)">Safari</div>
        </div>
      </div>
      <h3 style="font-size:13px;font-weight:600;margin-bottom:8px">Import History</h3>
      <div style="overflow-x:auto">
        <table>
          <thead><tr><th>Source File</th><th>Last Backup</th></tr></thead>
          <tbody>
            ${s.import_records.map(r => `
              <tr>
                <td style="font-family:monospace;font-size:12px">${r.data_path}</td>
                <td style="font-size:12px;color:var(--text-muted)">${fmtDate(r.last_import * 1000)}</td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      </div>
    `
  } catch (e) {
    document.getElementById('status-content').innerHTML = `<p style="color:red">Failed to load: ${e.message}</p>`
  }
}

loadStatus()
document.getElementById('refresh-status').addEventListener('click', loadStatus)

// Backup
let pollTimer = null

document.getElementById('start-backup').addEventListener('click', async () => {
  const files = document.getElementById('extra-files').value
    .split('\n').map(s => s.trim()).filter(Boolean)
  const disable_detect = document.getElementById('disable-detect').checked
  const dry_run = document.getElementById('dry-run').checked

  document.getElementById('start-backup').disabled = true
  document.getElementById('backup-progress').style.display = 'block'
  document.getElementById('backup-log').textContent = ''
  document.getElementById('backup-summary').style.display = 'none'

  const badge = document.getElementById('backup-status-badge')
  const statusText = document.getElementById('backup-status-text')
  badge.textContent = 'running'
  badge.style.background = 'var(--accent)'
  statusText.textContent = 'Backup in progress...'

  const res = await fetch('/api/backup', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ files, disable_detect, dry_run }),
  })
  const { job_id } = await res.json()

  let lastLineCount = 0
  pollTimer = setInterval(async () => {
    const poll = await fetch(`/api/backup/${job_id}`)
    const data = await poll.json()

    const logEl = document.getElementById('backup-log')
    if (data.log_lines.length > lastLineCount) {
      logEl.textContent = data.log_lines.join('\n')
      logEl.scrollTop = logEl.scrollHeight
      lastLineCount = data.log_lines.length
    }

    if (data.status === 'done' || data.status === 'error') {
      clearInterval(pollTimer)
      document.getElementById('start-backup').disabled = false
      badge.textContent = data.status
      badge.style.background = data.status === 'done' ? '#16a34a' : '#dc2626'
      statusText.textContent = data.status === 'done' ? 'Backup completed.' : 'Backup failed.'

      if (data.summary) {
        const s = data.summary
        const summaryEl = document.getElementById('backup-summary')
        summaryEl.style.display = 'block'
        summaryEl.innerHTML = s.error
          ? `<span style="color:red">Error: ${s.error}</span>`
          : `Found: <strong>${s.found}</strong> &nbsp; Imported: <strong>${s.imported}</strong> &nbsp; Duplicates: <strong>${s.duplicated}</strong>`
      }

      if (data.status === 'done') loadStatus()
    }
  }, 1000)
})
```

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build
```

Expected: no errors, `dist/db.html` present.

- [ ] **Step 4: Commit**

```bash
git add frontend/pages/db.html frontend/src/db.js
git commit -m "feat: build DB management page with status and async backup"
```

---

## Task 10: Wire frontend dist into Rust and update embed folder

**Files:**
- Modify: `src/web.rs`
- Modify: `.gitignore`

- [ ] **Step 1: Change the RustEmbed folder in `src/web.rs`**

```rust
// Change:
#[derive(RustEmbed)]
#[folder = "static"]
struct Asset;

// To:
#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;
```

- [ ] **Step 2: Update static file route prefix**

In the `serve()` method, the static route currently serves files under `/static/...`. Vite outputs assets to `frontend/dist/assets/`. Update the route:

```rust
// Change:
let static_route = warp::path("static")
    .and(warp::path::tail())
    .and_then(serve_file);

// To:
let assets_route = warp::path("assets")
    .and(warp::path::tail())
    .and_then(serve_file);
```

And update the routes chain: replace `static_route` with `assets_route`.

- [ ] **Step 3: Update `.gitignore`**

```
frontend/dist/
frontend/node_modules/
```

- [ ] **Step 4: Build both frontend and Rust, run and smoke-test**

```bash
cd frontend && npm run build && cd ..
cargo build
cargo run -- serve
```

Open http://127.0.0.1:9960 — verify index page loads with KPI cards, charts, dark/light toggle works, List button links to `/search`, DB link goes to `/db`.

- [ ] **Step 5: Commit**

```bash
git add src/web.rs .gitignore
git commit -m "feat: switch RustEmbed to frontend/dist, update asset route"
```

---

## Task 11: Delete static/ and update CI

**Files:**
- Delete: `static/` (entire directory)
- Modify: `.github/workflows/release.yml`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Verify old static/ is no longer referenced**

```bash
grep -r "static/" src/ --include="*.rs"
```

Expected: no matches (the embed folder has been updated in Task 10).

- [ ] **Step 2: Delete the static/ directory**

```bash
rm -rf static/
```

- [ ] **Step 3: Add `npm run build` step to CI in `.github/workflows/release.yml`**

Add this step before the `Build` step in the matrix job:

```yaml
      - name: Build frontend
        run: |
          cd frontend
          npm ci
          npm run build
```

- [ ] **Step 4: Update `CLAUDE.md` build commands section**

Replace the build commands block with:

```markdown
## Commands

```bash
# Frontend (run from frontend/)
npm install          # install dependencies (first time)
npm run build        # build to frontend/dist/ (required before cargo build)
npm run dev          # dev server on :5173 with proxy to :9960

# Rust
cargo build                                  # build (requires frontend/dist/ to exist)
cargo test                                   # run all tests
cargo test <test_name>                       # run single test
cargo clippy --all-targets --all-features    # lint
cargo fmt -- --check                         # check formatting
cargo run -- serve                           # run server (http://127.0.0.1:9960)
cargo run -- show                            # show detected browser history files
```

Full workflow: `cd frontend && npm run build && cd .. && cargo run -- serve`
```

- [ ] **Step 5: Final full build and smoke-test**

```bash
cd frontend && npm run build && cd ..
cargo build
cargo run -- serve
```

Verify: index `/`, search `/search`, details `/details/YYYY-MM-DD`, db `/db` all load correctly with new UI.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: remove legacy static/ dir, update CI and CLAUDE.md for two-step build"
```

---

## Self-Review Notes

**Spec coverage check:**

| Spec requirement | Task |
|---|---|
| Vite + Tailwind + ECharts 5 + flatpickr | Task 1, 3, 6 |
| Theme system (light/dark/auto + toggle) | Task 2 |
| KPI card row on index | Task 4, 6 |
| Daily PV + two pie charts side-by-side | Task 3, 6 |
| TOP 100 tables side-by-side | Task 6 |
| Domain chart click → search | Task 3, 6 |
| Search page virtual scroll | Task 7 |
| Search page client-side filter | Task 7 |
| Details page restyled | Task 8 |
| DB page: status section | Task 4, 9 |
| DB page: per-browser counts | Task 4, 9 |
| DB page: import records table | Task 4, 9 |
| DB page: backup form (files, disable-detect, dry-run) | Task 9 |
| Async backup with polling | Task 5, 9 |
| Backup progress log + summary | Task 5, 9 |
| Refresh status after backup done | Task 9 |
| `frontend/dist/` in `.gitignore` | Task 10 |
| CI: `npm run build` before `cargo build` | Task 11 |
| Delete `static/` | Task 11 |
| CLAUDE.md updated | Task 11 |
