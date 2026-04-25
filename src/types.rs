use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use warp::reject::Reject;

#[derive(Debug, Clone, Copy)]
pub enum SourceName {
    Safari,
    Firefox,
    Chrome,
}

#[derive(Debug, Serialize)]
pub struct VisitDetail {
    pub url: String,
    pub title: String,
    // unix_epoch_ms
    pub visit_time: i64,
    pub visit_type: i64,
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub total_visits: i64,
    pub unique_urls: i64,
    pub active_days: i64,
    pub today_visits: i64,
}

#[derive(Debug, Serialize)]
pub struct ImportRecord {
    pub data_path: String,
    pub last_import: i64,
}

#[derive(Debug, Serialize)]
pub struct DbStatus {
    pub file_path: String,
    pub file_size_bytes: u64,
    pub total_visits: i64,
    pub min_time: i64,
    pub max_time: i64,
    pub import_records: Vec<ImportRecord>,
}

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
    pub status: String,
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

#[derive(Debug, Deserialize)]
pub struct DetailsQueryParams {
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub start: Option<String>,
    pub end: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IndexQueryParams {
    pub start: Option<String>, // Y-m-d
    pub end: Option<String>,   // Y-m-d
    pub keyword: Option<String>,
}

#[derive(Debug)]
pub struct ServerError {
    pub e: String,
}

impl From<Error> for ServerError {
    fn from(err: Error) -> Self {
        Self {
            e: format!("{err:#}"),
        }
    }
}

impl Reject for ServerError {}

#[derive(Debug)]
pub struct ClientError {
    pub e: String,
}

impl From<Error> for ClientError {
    fn from(err: Error) -> Self {
        Self {
            e: format!("{err:#}"),
        }
    }
}

impl Reject for ClientError {}

#[derive(Serialize)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
}
