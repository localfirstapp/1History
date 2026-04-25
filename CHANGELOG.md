# Changelog

## v0.4.0 (2026-04-25)

### New Features

- Complete UI redesign: replaced Bootstrap 2 + jQuery with Vite 6 + Tailwind CSS 4 + ECharts 5
- Dark / light theme with automatic system detection and manual toggle
- KPI cards on the dashboard showing total visits, unique domains, active days, and today's count
- New `/search` page with virtual scroll — handles 10k+ records without lag, with client-side instant filter
- New `/db` (Database) page: database status, per-browser visit breakdown, import history, and in-browser backup with live progress
- Backup via web UI supports custom history file paths, disable auto-detect, and dry-run mode
- Case-insensitive search by default (toggle available in navbar)
- Chronological search results accessible from the List button in navbar

### Bug Fixes

- Fix end date being exclusive when passed as a query parameter (searches now include the full end day)

## v0.3.5 (2026-04-24)

### Bug Fixes

- Add Firefox snap flavor to supported default browser profiles ([#24](https://github.com/localfirstapp/1History/pull/24))

## v0.3.4 (2024-03-25)

### Bug Fixes

- Fix log not working

## v0.3.3 (2022-04-20)

### Bug Fixes

- If `The database file is locked` error arises during backup, copy the original SQLite db to a temp file and retry ([#19](https://github.com/localfirstapp/1History/pull/19))

## v0.3.2 (2022-04-18)

### Bug Fixes

- Fix CSV writer handling of special characters ([#16](https://github.com/localfirstapp/1History/pull/16))

## v0.3.1 (2022-08-14)

### New Features

- Add Flatpak variant browser detection

## v0.3.0 (2022-06-27)

### New Features

- Add progress bar when backing up
