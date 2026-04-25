use crate::{
    backup::backup_to_log,
    database::Database,
    types::{
        BackupJobResponse, BackupPollResponse, BackupRequest, BackupSummary, ClientError,
        DetailsQueryParams, ErrorMessage, IndexQueryParams, SearchQueryParams, ServerError,
    },
    util::{
        detect_history_files, full_timerange, minijinja_format_as_hms, minijinja_format_as_ymd,
        minijinja_format_as_ymdhms, minijinja_format_title, tomorrow_midnight, ymd_midnight,
    },
};
use anyhow::{Context, Error, Result};
use log::{error, info, warn};
use minijinja::{Environment, context};
use rust_embed::RustEmbed;
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;
use uuid::Uuid;
use warp::{
    Filter, Rejection, Reply,
    http::HeaderValue,
    hyper::StatusCode,
    path::Tail,
    reject,
    reply::{self, Response},
};

const DEFAULT_SEARCH_INTERVAL: i64 = 3_600_000 * 24 * 30; // 30 days

#[derive(RustEmbed)]
#[folder = "static"]
struct Asset;

async fn serve_file(path: Tail) -> Result<impl Reply, Rejection> {
    let path = path.as_str();
    let asset = Asset::get(path).ok_or_else(reject::not_found)?;
    let mut res = Response::new(asset.data.into());

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    if let Ok(v) = HeaderValue::from_str(mime.as_ref()) {
        res.headers_mut().insert("content-type", v);
    }

    Ok(res)
}

type JobId = String;

const JOB_TTL_SECS: u64 = 3600; // remove finished jobs after 1 hour

#[derive(Clone)]
struct BackupJob {
    log_lines: Arc<Mutex<Vec<String>>>,
    status: Arc<Mutex<String>>,
    summary: Arc<Mutex<Option<BackupSummary>>>,
    finished_at: Arc<Mutex<Option<std::time::Instant>>>,
}

type JobStore = Arc<Mutex<HashMap<JobId, BackupJob>>>;

struct Server {
    db: Arc<Database>,
    addr: SocketAddr,
    db_file: String,
    jobs: JobStore,
}

impl Server {
    fn try_new(addr: String, db_filepath: String) -> Result<Self> {
        Ok(Self {
            db: Arc::new(Database::open(db_filepath.clone()).context("open db")?),
            addr: addr.parse()?,
            db_file: db_filepath,
            jobs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn with_db(
        db: Arc<Database>,
    ) -> impl Filter<Extract = (Arc<Database>,), Error = Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn with_jobs(jobs: JobStore) -> impl Filter<Extract = (JobStore,), Error = Infallible> + Clone {
        warp::any().map(move || jobs.clone())
    }

    fn with_db_file(
        db_file: String,
    ) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
        warp::any().map(move || db_file.clone())
    }

    async fn details(
        db: Arc<Database>,
        ymd: String,
        query_params: DetailsQueryParams,
    ) -> Result<impl Reply, Rejection> {
        let start = ymd_midnight(&ymd).map_err(ClientError::from)?;
        let end = start + 3_600_000 * 24;
        let keyword = query_params.keyword;
        let visit_details = db
            .select_visits(start, end, keyword.clone())
            .map_err(ServerError::from)?;

        let asset = Asset::get("details.html").unwrap();
        let index_tmpl: &str =
            std::str::from_utf8(&asset.data).map_err(|e| ServerError::from(Error::from(e)))?;
        let mut env = Environment::new();
        env.add_template("details", index_tmpl)
            .map_err(|e| ServerError::from(Error::from(e)))?;

        env.add_function("format_as_ymd", minijinja_format_as_ymd);
        env.add_function("format_as_hms", minijinja_format_as_hms);
        env.add_function("format_title", minijinja_format_title);
        let tmpl = env.get_template("details").unwrap();
        let body = tmpl
            .render(context!(
                ymd => ymd,
                ymd_ts => start,
                visit_details => visit_details,
                version => clap::crate_version!(),
                keyword => keyword.unwrap_or_default(),
            ))
            .map_err(|e| ServerError::from(Error::from(e)))?;

        Ok(reply::html(body))
    }

    async fn index(
        db: Arc<Database>,
        query_params: IndexQueryParams,
    ) -> Result<impl Reply, Rejection> {
        let end = query_params
            .end
            .map_or_else(|| Ok(tomorrow_midnight() - 1), |ymd| ymd_midnight(&ymd).map(|t| t + 86_400_000 - 1))
            .map_err(ClientError::from)?;
        let start = query_params
            .start
            .map_or_else(
                || Ok(tomorrow_midnight() - DEFAULT_SEARCH_INTERVAL),
                |ymd| ymd_midnight(&ymd),
            )
            .map_err(ClientError::from)?;
        let keyword = query_params.keyword;
        

        let daily_counts = db
            .select_daily_count(start, end, keyword.clone())
            .context("daily_count")
            .map_err(ServerError::from)?;
        let (min_time, max_time) = match db.select_min_max_time() {
            Ok(v) => v,
            Err(e) => {
                warn!("Select min_max time failed and fallback to default, msg:{e:?}");
                full_timerange()
            }
        };

        let title_top100 = db
            .select_title_top100(start, end, keyword.clone())
            .context("title_top100")
            .map_err(ServerError::from)?;
        let domain_top100 = db
            .select_domain_top100(start, end, keyword.clone())
            .context("domain_top100")
            .map_err(ServerError::from)?;
        let visit_details = db
            .select_visits(start, end, keyword.clone())
            .context("visit_details")
            .map_err(ServerError::from)?;
        let stats = db
            .select_stats(start, end)
            .context("stats")
            .map_err(ServerError::from)?;

        let asset = Asset::get("index.html").unwrap();
        let index_tmpl: &str =
            std::str::from_utf8(&asset.data).map_err(|e| ServerError::from(Error::from(e)))?;
        let mut env = Environment::new();
        env.add_template("index", index_tmpl)
            .map_err(|e| ServerError::from(Error::from(e)))?;
        env.add_function("format_as_ymdhms", minijinja_format_as_ymdhms);
        env.add_function("format_title", minijinja_format_title);
        let tmpl = env.get_template("index").unwrap();
        let body = tmpl
            .render(context!(
                min_time => min_time,
                max_time => max_time,
                start => start,
                end => end,
                daily_counts => daily_counts,
                title_top100 => title_top100,
                domain_top100 => domain_top100,
                visit_details => visit_details,
                stats => stats,
                start_ymd => crate::util::unixepoch_as_ymd(start),
                end_ymd => crate::util::unixepoch_as_ymd(end),
                keyword => keyword.unwrap_or_default(),
                
                version => clap::crate_version!(),
            ))
            .map_err(|e| ServerError::from(Error::from(e)))?;

        Ok(reply::html(body))
    }

    async fn search(
        db: Arc<Database>,
        query_params: SearchQueryParams,
    ) -> Result<impl Reply, Rejection> {
        let end = query_params
            .end
            .map_or_else(|| Ok(tomorrow_midnight() - 1), |ymd| ymd_midnight(&ymd).map(|t| t + 86_400_000 - 1))
            .map_err(ClientError::from)?;
        let start = query_params
            .start
            .map_or_else(
                || Ok(tomorrow_midnight() - DEFAULT_SEARCH_INTERVAL),
                |ymd| ymd_midnight(&ymd),
            )
            .map_err(ClientError::from)?;
        let keyword = query_params.keyword;
        

        let visit_details = db
            .select_visits(start, end, keyword.clone())
            .context("visit_details")
            .map_err(ServerError::from)?;

        let asset = Asset::get("search.html").unwrap();
        let tmpl_str: &str =
            std::str::from_utf8(&asset.data).map_err(|e| ServerError::from(Error::from(e)))?;
        let mut env = Environment::new();
        env.add_template("search", tmpl_str)
            .map_err(|e| ServerError::from(Error::from(e)))?;
        env.add_function("format_as_ymdhms", minijinja_format_as_ymdhms);
        env.add_function("format_title", minijinja_format_title);
        let tmpl = env.get_template("search").unwrap();
        let body = tmpl
            .render(context!(
                start_ymd => crate::util::unixepoch_as_ymd(start),
                end_ymd => crate::util::unixepoch_as_ymd(end),
                visit_details => visit_details,
                keyword => keyword.unwrap_or_default(),
                version => clap::crate_version!(),
            ))
            .map_err(|e| ServerError::from(Error::from(e)))?;

        Ok(reply::html(body))
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
        let log_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new("running".to_string()));
        let summary_store: Arc<Mutex<Option<BackupSummary>>> = Arc::new(Mutex::new(None));

        let finished_at: Arc<Mutex<Option<std::time::Instant>>> = Arc::new(Mutex::new(None));
        let job = BackupJob {
            log_lines: Arc::clone(&log_lines),
            status: Arc::clone(&status),
            summary: Arc::clone(&summary_store),
            finished_at: Arc::clone(&finished_at),
        };

        // evict finished jobs older than TTL before inserting a new one
        {
            let mut store = jobs.lock().unwrap();
            store.retain(|_, j| {
                j.finished_at.lock().unwrap()
                    .map_or(true, |t| t.elapsed().as_secs() < JOB_TTL_SECS)
            });
            store.insert(job_id.clone(), job);
        }

        tokio::task::spawn_blocking(move || {
            let mut files = if req.disable_detect {
                vec![]
            } else {
                detect_history_files()
            };
            files.extend(req.files);
            match backup_to_log(files, db_file, req.dry_run, Arc::clone(&log_lines)) {
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
                        found: 0,
                        imported: 0,
                        duplicated: 0,
                        error: Some(format!("{e:?}")),
                    });
                    *status.lock().unwrap() = "error".to_string();
                }
            }
            *finished_at.lock().unwrap() = Some(std::time::Instant::now());
        });

        Ok(warp::reply::json(&BackupJobResponse { job_id }))
    }

    async fn poll_backup(job_id: String, jobs: JobStore) -> Result<impl Reply, Rejection> {
        let jobs = jobs.lock().unwrap();
        let job = jobs.get(&job_id).ok_or_else(|| {
            warp::reject::custom(ClientError {
                e: "job not found".to_string(),
            })
        })?;
        let status = job.status.lock().unwrap().clone();
        let log_lines = job.log_lines.lock().unwrap().clone();
        let summary = job.summary.lock().unwrap().clone();
        Ok(warp::reply::json(&BackupPollResponse {
            status,
            log_lines,
            summary,
        }))
    }

    // https://github.com/ItsNothingPersonal/warp-postgres-example/blob/main/src/main.rs#L63
    fn serve(&self) -> Result<()> {
        let index = warp::path::end()
            .and(Self::with_db(self.db.clone()))
            .and(warp::query::<IndexQueryParams>())
            .and_then(Self::index);

        let detail = Self::with_db(self.db.clone())
            .and(warp::path!("details" / String))
            .and(warp::query::<DetailsQueryParams>())
            .and_then(Self::details);

        let search = Self::with_db(self.db.clone())
            .and(warp::path!("search"))
            .and(warp::query::<SearchQueryParams>())
            .and_then(Self::search);

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

        let db_page = warp::path("db").and(warp::path::end()).map(|| {
            let asset = Asset::get("db.html").unwrap();
            reply::html(String::from_utf8_lossy(&asset.data).to_string())
        });

        let static_route = warp::path("static")
            .and(warp::path::tail())
            .and_then(serve_file);

        let routes = detail
            .or(search)
            .or(db_status_route)
            .or(backup_start)
            .or(backup_poll)
            .or(db_page)
            .or(index)
            .or(static_route)
            .recover(Self::handle_rejection);

        let rt = Runtime::new().context("tokio runtime build")?;
        rt.block_on(async {
            info!("Start HTTP server on {}...", self.addr);
            warp::serve(routes).run(self.addr).await;
        });
        Ok(())
    }

    async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
        let code;
        let message;

        if err.is_not_found() {
            code = StatusCode::NOT_FOUND;
            message = "NOT_FOUND";
        } else if let Some(ServerError { e }) = err.find() {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = e;
        } else if let Some(ClientError { e }) = err.find() {
            code = StatusCode::BAD_REQUEST;
            message = e;
        } else {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            message = "UNHANDLED_REJECTION";
        }

        let json = warp::reply::json(&ErrorMessage {
            message: message.into(),
            code: code.as_u16(),
        });

        if code != 404 {
            error!("{err:?}");
        }

        Ok(warp::reply::with_status(json, code))
    }
}

pub fn serve(addr: String, db_filepath: String) -> Result<()> {
    let server = Server::try_new(addr, db_filepath)?;
    server.serve()
}
