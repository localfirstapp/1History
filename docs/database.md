# Database Design

1History uses a single SQLite file (default `~/onehistory.db`) to store all browser history.

## Schema

### `onehistory_urls`

Stores the URL and title for each unique page.

```sql
CREATE TABLE onehistory_urls (
    id        integer PRIMARY KEY AUTOINCREMENT,
    url       text NOT NULL UNIQUE,
    title     text
);
```

| Column | Description |
|--------|-------------|
| `id` | Auto-incremented primary key |
| `url` | Full URL, unique across all browsers |
| `title` | Page title at the time of the last visit (may be empty) |

### `onehistory_visits`

Records each visit event, linked to a URL by `item_id`.

```sql
CREATE TABLE onehistory_visits (
    id         integer PRIMARY KEY AUTOINCREMENT,
    item_id    integer,
    visit_time integer,
    visit_type integer NOT NULL DEFAULT 0,
    UNIQUE(item_id, visit_time)
);
```

| Column | Description |
|--------|-------------|
| `item_id` | Foreign key → `onehistory_urls.id` |
| `visit_time` | Visit timestamp in **microseconds** (PRTime / Firefox epoch — see below) |
| `visit_type` | Browser-specific visit type code, passed through as-is from the source browser (see below) |

The `UNIQUE(item_id, visit_time)` constraint is what prevents duplicate visits when the same history file is backed up multiple times.

### `import_records`

Tracks which history files have been imported and when.

```sql
CREATE TABLE import_records (
    id          integer PRIMARY KEY AUTOINCREMENT,
    last_import integer,
    data_path   text NOT NULL UNIQUE
);
```

| Column | Description |
|--------|-------------|
| `data_path` | Absolute path of the source history file (original path, never a temp file) |
| `last_import` | Timestamp of the most recent backup from this file, in microseconds |

## visit_type Values

`visit_type` is stored as-is from the source browser and has different meanings per browser:

| Browser | Source field | Meaning |
|---------|-------------|---------|
| Firefox | `moz_historyvisits.visit_type` | 1=link, 2=typed, 3=bookmark, 4=embed, 5=redirect (permanent), 6=redirect (temporary), 7=download, 8=framed link |
| Chrome | `visits.transition & 0xFF` | 0=link, 1=typed, 2=auto bookmark, 3=auto subframe, 4=manual subframe, 7=form submit, 8=reload |
| Safari | — | Always `-1` (Safari does not expose a visit type) |

## Timestamp Format

All timestamps in the database are stored in **PRTime** (Firefox epoch): microseconds since **1970-01-01 00:00:00 UTC**.

Each browser uses a different native timestamp format, which is normalized to PRTime during import:

| Browser | Native format | Epoch | Conversion |
|---------|--------------|-------|------------|
| Firefox | PRTime (µs) | 1970-01-01 | — (already PRTime) |
| Chrome / Chromium | WebKit (µs) | 1601-01-01 | subtract 11644473600 × 10⁶ |
| Safari | NSDate (seconds, float) | 2001-01-01 | add 978307200, multiply × 10⁶ |

When querying, the Rust layer converts Unix milliseconds (used internally by the app) to PRTime via `ts * 1000`, and converts stored `visit_time` back to Unix milliseconds with `visit_time / 1000`.

## Deduplication

Duplicate visits are silently ignored at insert time via the `UNIQUE(item_id, visit_time)` constraint. This means it is always safe to run `backup` multiple times on the same history file.
