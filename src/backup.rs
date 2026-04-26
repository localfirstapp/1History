use std::fs;
use std::sync::{Arc, Mutex};

use crate::progress::{LogCollector, ProgressCollector, TUICollector};
use crate::source::Source;
use crate::types::{SourceName, VisitDetail};
use crate::{database::Database, util::full_timerange};
use anyhow::{Context, Error, Result};
use log::{debug, info};

pub struct BackupResult {
    pub found: usize,
    pub imported: usize,
    pub duplicated: usize,
    pub failed: usize,
}

fn load_rows(path: &str, start: i64, end: i64) -> Result<(SourceName, Vec<VisitDetail>)> {
    let s = Source::open(path).context("open")?;
    let name = s.name();
    let rows = s.select(start, end).context("select")?.collect::<Vec<_>>();
    Ok((name, rows))
}

fn load_rows_from_temp_copy(
    path: &str,
    start: i64,
    end: i64,
) -> Result<(SourceName, Vec<VisitDetail>)> {
    let tmp = tempfile::NamedTempFile::new().context("create temp file")?;
    fs::copy(path, tmp.path()).with_context(|| format!("copy source file {path} to temp"))?;
    let tmp_path = tmp.into_temp_path();
    let tmp_str = tmp_path.to_string_lossy().to_string();
    load_rows(&tmp_str, start, end)
}

enum BackupCollector {
    Tui(TUICollector),
    Log(LogCollector),
}

impl ProgressCollector for BackupCollector {
    fn inc(&self, delta: u64) {
        match self {
            Self::Tui(c) => c.inc(delta),
            Self::Log(c) => c.inc(delta),
        }
    }

    fn finish(&self) {
        match self {
            Self::Tui(c) => c.finish(),
            Self::Log(c) => c.finish(),
        }
    }
}

fn push_log(log_lines: &Option<Arc<Mutex<Vec<String>>>>, msg: String) {
    if let Some(lines) = log_lines {
        lines.lock().unwrap().push(msg);
    }
}

pub fn backup(
    history_files: Vec<String>,
    db_file: String,
    dry_run: bool,
    log_lines: Option<Arc<Mutex<Vec<String>>>>,
) -> Result<BackupResult> {
    let (start, end) = full_timerange();
    debug!("files:{history_files:?}, start:{start}, end:{end}");
    let db = Database::open(db_file).context("open 1History DB")?;

    let mut found = 0;
    let mut total_affected = 0;
    let mut total_duplicated = 0;
    let mut failed = 0;

    let mut persist = |name: SourceName, record_path: &str, rows: Vec<VisitDetail>| {
        let row_count = rows.len();
        found += row_count;
        let collector = if let Some(lines) = &log_lines {
            push_log(
                &log_lines,
                format!("Processing {} ({} records)...", record_path, row_count),
            );
            BackupCollector::Log(LogCollector::new(
                record_path.to_string(),
                row_count as u64,
                Arc::clone(lines),
            ))
        } else {
            info!("Begin backup {}...", record_path);
            BackupCollector::Tui(TUICollector::new(row_count as u64))
        };
        if !dry_run {
            let (affected, duplicated) = db
                .persist(record_path, rows, collector)
                .context("persist")?;
            total_affected += affected;
            total_duplicated += duplicated;
            debug!("{name:?} affected:{affected}, duplicated:{duplicated}");
        }
        debug!("{name:?} select {row_count} histories");
        if log_lines.is_some() {
            push_log(&log_lines, format!("Done: {}", record_path));
        } else {
            info!("Finish backup {}", record_path);
        }
        Ok::<_, Error>(())
    };

    for his_file in &history_files {
        let loaded = match load_rows(his_file, start, end) {
            Ok(loaded) => Ok(loaded),
            Err(e) => {
                debug!("Direct source read failed for {his_file}: {e:?}");
                load_rows_from_temp_copy(his_file, start, end).map_err(|retry_err| (e, retry_err))
            }
        };

        match loaded {
            Ok((name, rows)) => {
                if let Err(e) = persist(name, his_file, rows) {
                    failed += 1;
                    let msg = if log_lines.is_some() {
                        format!("Skip {}: {e:#}", his_file)
                    } else {
                        format!("Skip backup {}: {e:#}", his_file)
                    };
                    info!("{msg}");
                    push_log(&log_lines, msg);
                }
            }
            Err((direct_err, retry_err)) => {
                failed += 1;
                let msg = if log_lines.is_some() {
                    format!(
                        "Skip {}: direct read failed: {direct_err:#}; temp copy retry failed: {retry_err:#}",
                        his_file
                    )
                } else {
                    format!(
                        "Skip backup {}: direct read failed: {direct_err:#}; temp copy retry failed: {retry_err:#}",
                        his_file
                    )
                };
                info!("{msg}");
                push_log(&log_lines, msg);
            }
        }
    }

    let result = BackupResult {
        found,
        imported: total_affected,
        duplicated: total_duplicated,
        failed,
    };
    if log_lines.is_none() {
        info!(
            "Summary\nFound:{}, Imported:{}, Duplicated: {}, Failed:{}",
            result.found, result.imported, result.duplicated, result.failed
        );
    }
    Ok(result)
}
