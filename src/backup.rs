use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::progress::{LogCollector, TUICollector};
use crate::source::Source;
use crate::{database::Database, util::full_timerange};
use anyhow::{Context, Error, Result};
use log::{debug, info};

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
) -> Result<BackupResult> {
    let (start, end) = full_timerange();
    let db = Database::open(db_file).context("open 1History DB")?;

    let mut found = 0;
    let mut total_affected = 0;
    let mut total_duplicated = 0;

    // open_path: file to read from; record_path: path stored in import_records
    let mut persist = |open_path: &str, record_path: &str| {
        let s = Source::open(open_path).context("open")?;
        let rows = s.select(start, end).context("select")?.collect::<Vec<_>>();
        found += rows.len();
        log_lines.lock().unwrap().push(format!(
            "Processing {} ({} records)...",
            record_path,
            rows.len()
        ));
        let collector = LogCollector::new(
            record_path.to_string(),
            rows.len() as u64,
            Arc::clone(&log_lines),
        );
        if !dry_run {
            let (affected, duplicated) =
                db.persist(record_path, rows, collector).context("persist")?;
            total_affected += affected;
            total_duplicated += duplicated;
        }
        log_lines
            .lock()
            .unwrap()
            .push(format!("Done: {}", record_path));
        Ok::<_, Error>(())
    };

    for his_file in &history_files {
        if let Err(e) = persist(his_file, his_file) {
            let msg = format!("{e:?}");
            if msg.contains("The database file is locked") {
                let mut tmp = match tempfile::NamedTempFile::new() {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let body = match fs::read(his_file) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if tmp.write_all(&body).is_err() {
                    continue;
                }
                let tmp_path = tmp.into_temp_path();
                let tmp_str = tmp_path.to_string_lossy().to_string();
                // use his_file as record_path so import_records shows the real path
                let _ = persist(&tmp_str, his_file);
            }
        }
    }

    Ok(BackupResult {
        found,
        imported: total_affected,
        duplicated: total_duplicated,
    })
}

pub fn backup(history_files: Vec<String>, db_file: String, dry_run: bool) -> Result<()> {
    let (start, end) = full_timerange();
    debug!("files:{history_files:?}, start:{start}, end:{end}");

    let db = Database::open(db_file).context("open 1History DB")?;

    let mut found = 0;
    let mut total_affected = 0;
    let mut total_duplicated = 0;
    let mut persist = |open_path: &str, record_path: &str| {
        let s = Source::open(open_path).context("open")?;
        let rows = s.select(start, end).context("select")?.collect::<Vec<_>>();
        debug!("{:?} select {} histories", s.name(), rows.len());
        found += rows.len();

        info!("Begin backup {}...", record_path);
        let collector = TUICollector::new(rows.len() as u64);
        if !dry_run {
            let (affected, duplicated) =
                db.persist(record_path, rows, collector).context("persist")?;
            debug!(
                "{:?} affected:{}, duplicated:{}",
                s.name(),
                affected,
                duplicated
            );
            total_affected += affected;
            total_duplicated += duplicated;
        };
        info!("Finish backup {}", record_path);

        Ok::<_, Error>(())
    };
    for his_file in &history_files {
        if let Err(e) = persist(his_file, his_file) {
            let msg = format!("{e:?}");
            if msg.contains("The database file is locked") {
                debug!("Open database directly failed, copy to temp and backup again");
                let mut tmp = match tempfile::NamedTempFile::new() {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let body = match fs::read(his_file) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Err(e) = tmp.write_all(&body) {
                    debug!("Copy to backup file failed, msg:{e}");
                    continue;
                }
                let tmp_path = tmp.into_temp_path();
                let tmp_str = tmp_path.to_string_lossy().to_string();
                if let Err(e) = persist(&tmp_str, his_file) {
                    debug!("{his_file} persist failed, backup:{tmp_str}, err: {e:?}");
                }
            }
        }
    }

    info!("Summary\nFound:{found}, Imported:{total_affected}, Duplicated: {total_duplicated}");
    Ok(())
}
