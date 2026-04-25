use crate::{
    progress::ProgressCollector,
    types::VisitDetail,
    util::{domain_from, ymd_midnight},
};
use anyhow::{Context, Result};
use log::debug;
use rusqlite::{Connection, Error as sqlError, ErrorCode, Transaction, named_params};
use std::{collections::HashMap, sync::Mutex};

#[derive(Debug)]
struct HistoryVisit {
    item_id: i64,
    visit_time: i64,
    visit_type: i64,
}

const DEFAULT_BATCH_NUM: usize = 100;

pub(crate) struct Database {
    conn: Mutex<Connection>,
    persist_batch: usize,
    pub file_path: String,
}

impl Database {
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

    fn init(&self) -> Result<()> {
        self.conn
            .lock()
            .unwrap()
            .execute_batch(
                r#"
CREATE TABLE IF NOT EXISTS onehistory_urls (
    id integer PRIMARY KEY AUTOINCREMENT,
    url text NOT NULL UNIQUE,
    title text
);

CREATE TABLE IF NOT EXISTS onehistory_visits (
    id integer PRIMARY KEY AUTOINCREMENT,
    item_id integer,
    visit_time integer,
    visit_type integer NOT NULL DEFAULT 0,
    UNIQUE(item_id, visit_time)
);


CREATE TABLE IF NOT EXISTS import_records (
    id integer PRIMARY KEY AUTOINCREMENT,
    last_import integer,
    data_path text NOT NULL UNIQUE);
"#,
            )
            .context("create table")?;
        Ok(())
    }

    fn get_or_persist_url(&self, url: String, title: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let query_id = || -> rusqlite::Result<i64> {
            let mut stat = conn.prepare(
                r#"
         SELECT id FROM "onehistory_urls" WHERE url = :url;
"#,
            )?;
            stat.query_row(
                named_params! {
                    ":url": url,
                },
                |row| row.get(0),
            )
        };
        match query_id() {
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let mut stat = conn.prepare(
                    r#"
    INSERT INTO "onehistory_urls" (url, title) VALUES(:url, :title);
"#,
                )?;
                let affected = stat
                    .execute(named_params! {
                        ":url": url,
                        ":title": title,
                    })
                    .context("insert onehistory_urls")?;
                assert_eq!(affected, 1);

                let id = query_id()?;
                Ok(id)
            }
            Err(e) => Err(e.into()),
            Ok(id) => Ok(id),
        }
    }

    fn persist_visits(&self, src_path: &str, batch: Vec<HistoryVisit>) -> Result<(usize, usize)> {
        assert!(!batch.is_empty());

        let sql = r#"
INSERT INTO onehistory_visits (item_id, visit_time, visit_type)
    VALUES (?1, ?2, ?3);
"#;

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let last_ts = batch[batch.len() - 1].visit_time;
        let mut affected = 0;
        let mut duplicated = 0;
        for HistoryVisit {
            item_id,
            visit_time,
            visit_type,
        } in batch
        {
            match tx.execute(sql, [&item_id, &visit_time, &visit_type]) {
                Ok(ret) => affected += ret,
                Err(e) => {
                    if let sqlError::SqliteFailure(ffi_err, _msg) = &e
                        && ffi_err.code == ErrorCode::ConstraintViolation
                    {
                        duplicated += 1;
                        let ext_code = ffi_err.extended_code;
                        debug!(
                            "[ignore]onehistory_visits duplicated. id:{item_id}, \
                                      time:{visit_time}, type:{visit_type}, ext_code:{ext_code}"
                        );
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
        Self::update_process(&tx, src_path, last_ts)?;
        tx.commit()?;

        Ok((affected, duplicated))
    }

    pub fn persist(
        &self,
        src_path: &str,
        details: Vec<VisitDetail>,
        collector: impl ProgressCollector,
    ) -> Result<(usize, usize)> {
        let mut i = 0;
        let mut batch = None; // Use Option so we can take it out later
        let mut affected = 0;
        let mut duplicated = 0;
        let mut persist_helper = |batch: Vec<HistoryVisit>| -> Result<()> {
            let len = batch.len();
            let (a, d) = self.persist_visits(src_path, batch)?;
            affected += a;
            duplicated += d;
            collector.inc(len as u64);
            Ok(())
        };
        for VisitDetail {
            url,
            title,
            visit_time,
            visit_type,
        } in details
        {
            i += 1;
            let one_batch = batch.get_or_insert(Vec::with_capacity(self.persist_batch));
            let item_id = self.get_or_persist_url(url, title)?;
            one_batch.push(HistoryVisit {
                item_id,
                visit_time,
                visit_type,
            });
            if i % self.persist_batch == 0 {
                persist_helper(batch.take().unwrap())?;
            }
        }
        if batch.is_some() {
            persist_helper(batch.take().unwrap())?;
        }
        collector.finish();

        Ok((affected, duplicated))
    }

    fn update_process(tx: &Transaction<'_>, src_path: &str, ts: i64) -> Result<()> {
        let sql = r#"
INSERT INTO import_records (last_import, data_path)
    VALUES (:last_import, :data_path)
ON CONFLICT (data_path)
    DO UPDATE SET
        last_import = :last_import;
"#;
        tx.execute(
            sql,
            named_params! {
                ":last_import": ts,
                ":data_path": src_path,
            },
        )?;

        Ok(())
    }

    fn unixepoch_to_prtime(ts: i64) -> i64 {
        ts * 1_000
    }

    // Returns (sql_fragment, bound_value).
    // When a keyword is present the fragment uses the :kw placeholder;
    // callers must bind it to the returned value.
    fn keyword_to_like(kw: Option<String>, case_sensitive: bool) -> (String, Option<String>) {
        match kw {
            None => ("1".to_string(), None),
            Some(v) => {
                let bound = format!("%{}%", v);
                let fragment = if case_sensitive {
                    "(url like :kw or title like :kw)".to_string()
                } else {
                    "(lower(url) like lower(:kw) or lower(title) like lower(:kw))".to_string()
                };
                (fragment, Some(bound))
            }
        }
    }

    pub fn select_visits(
        &self,
        start: i64,
        end: i64,
        keyword: Option<String>,
        case_sensitive: bool,
    ) -> Result<Vec<VisitDetail>> {
        let (kw_fragment, kw_value) = Self::keyword_to_like(keyword, case_sensitive);
        let sql = format!(
            r#"
SELECT
    url,
    title,
    CAST(visit_time / 1000 as integer),
    visit_type
FROM
    onehistory_urls u,
    onehistory_visits v ON u.id = v.item_id
WHERE
    visit_time BETWEEN :start AND :end and {kw_fragment}
ORDER BY
    visit_time
"#
        );

        let conn = self.conn.lock().unwrap();
        let mut stat = conn.prepare(&sql)?;

        let rows = stat.query_map(
            named_params! {
                ":start": Self::unixepoch_to_prtime(start),
                ":end": Self::unixepoch_to_prtime(end),
                ":kw": kw_value,
            },
            |row| {
                let detail = VisitDetail {
                    url: row.get(0)?,
                    title: row.get(1).unwrap_or_else(|_| "".to_string()),
                    visit_time: row.get(2)?,
                    visit_type: 0,
                };
                Ok(detail)
            },
        )?;

        let mut res: Vec<VisitDetail> = Vec::new();
        for r in rows {
            res.push(r?);
        }

        Ok(res)
    }

    pub fn select_daily_count(
        &self,
        start: i64,
        end: i64,
        keyword: Option<String>,
        case_sensitive: bool,
    ) -> Result<Vec<(i64, i64)>> {
        let (kw_fragment, kw_value) = Self::keyword_to_like(keyword, case_sensitive);
        let sql = format!(
            r#"
SELECT
    visit_day,
    count(1)
FROM (
    SELECT
        strftime ('%Y-%m-%d', visit_time / 1000000, 'unixepoch', 'localtime') AS visit_day
    FROM
        onehistory_visits v,
        onehistory_urls u ON v.item_id = u.id
    WHERE
        visit_time BETWEEN :start AND :end
        AND {kw_fragment})
    GROUP BY
        visit_day
    ORDER BY
        visit_day;
"#
        );
        debug!("Daily count sql: {sql}, start:{start}, end:{end}");
        let conn = self.conn.lock().unwrap();
        let mut stat = conn.prepare(&sql)?;

        let rows = stat.query_map(
            named_params! {
                ":start": Self::unixepoch_to_prtime(start),
                ":end": Self::unixepoch_to_prtime(end),
                ":kw": kw_value,
            },
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let mut res = Vec::new();
        for r in rows {
            let (ymd, cnt): (String, i64) = r?;
            res.push((ymd_midnight(&ymd)?, cnt));
        }

        Ok(res)
    }

    pub fn select_domain_top100(
        &self,
        start: i64,
        end: i64,
        keyword: Option<String>,
        case_sensitive: bool,
    ) -> Result<Vec<(String, i64)>> {
        let (kw_fragment, kw_value) = Self::keyword_to_like(keyword, case_sensitive);
        let sql = format!(
            r#"
SELECT
    url,
    count(1) AS cnt
FROM (
    SELECT
        url
    FROM
        onehistory_visits v,
        onehistory_urls u ON v.item_id = u.id
    WHERE
        visit_time BETWEEN :start AND :end
        AND title != '' AND {kw_fragment})
GROUP BY
    url
ORDER BY
    cnt DESC
"#
        );
        let url_top100 = self.select_top100(&sql, start, end, kw_value)?;

        let mut domain_top = HashMap::new();
        for (url, cnt) in url_top100 {
            let domain = domain_from(url);
            let total = domain_top.entry(domain).or_insert(cnt);
            *total += cnt;
        }
        let mut top_arr = domain_top.into_iter().collect::<Vec<(String, i64)>>();
        top_arr.sort_by_key(|b| std::cmp::Reverse(b.1));

        Ok(top_arr.into_iter().take(100).collect::<Vec<_>>())
    }

    pub fn select_title_top100(
        &self,
        start: i64,
        end: i64,
        keyword: Option<String>,
        case_sensitive: bool,
    ) -> Result<Vec<(String, i64)>> {
        let (kw_fragment, kw_value) = Self::keyword_to_like(keyword, case_sensitive);
        let sql = format!(
            r#"
SELECT
    title,
    count(1) AS cnt
FROM (
    SELECT
        title
    FROM
        onehistory_visits v,
        onehistory_urls u ON v.item_id = u.id
    WHERE
        visit_time BETWEEN :start AND :end
        AND title != '' AND {kw_fragment})
GROUP BY
    title
ORDER BY
    cnt DESC
LIMIT 100;
"#
        );
        self.select_top100(&sql, start, end, kw_value)
    }

    fn select_top100(&self, sql: &str, start: i64, end: i64, kw_value: Option<String>) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stat = conn.prepare(sql)?;

        let rows = stat.query_map(
            named_params! {
                ":start": Self::unixepoch_to_prtime(start),
                ":end": Self::unixepoch_to_prtime(end),
                ":kw": kw_value,
            },
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }

        Ok(res)
    }

    pub fn select_min_max_time(&self) -> Result<(i64, i64)> {
        let sql = r#"
SELECT
    CAST(min(visit_time) / 1000 AS integer),
    CAST(max(visit_time) / 1000 AS integer)
FROM
    onehistory_visits
"#;
        let conn = self.conn.lock().unwrap();
        let mut stat = conn.prepare(sql)?;

        let time_range = stat.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        Ok(time_range)
    }

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
            |row| {
                Ok(crate::types::Stats {
                    total_visits: row.get(0)?,
                    unique_domains: row.get(1)?,
                    active_days: row.get(2)?,
                    today_visits: row.get(3)?,
                })
            },
        )?;
        Ok(result)
    }

    pub fn select_db_status(&self) -> Result<crate::types::DbStatus> {
        use std::fs;
        let file_size_bytes = fs::metadata(&self.file_path).map(|m| m.len()).unwrap_or(0);
        let conn = self.conn.lock().unwrap();

        let (total_visits, min_time, max_time): (i64, i64, i64) = conn.query_row(
            "SELECT count(1), CAST(coalesce(min(visit_time),0)/1000 AS integer), CAST(coalesce(max(visit_time),0)/1000 AS integer) FROM onehistory_visits",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        let chrome_visits: i64 = conn
            .query_row(
                "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url NOT LIKE 'place%' AND u.url NOT LIKE 'about:%'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let firefox_visits: i64 = conn
            .query_row(
                "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url LIKE 'place%'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let safari_visits: i64 = conn
            .query_row(
                "SELECT count(1) FROM onehistory_visits v JOIN onehistory_urls u ON v.item_id = u.id WHERE u.url LIKE 'about:%'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let mut import_stat = conn.prepare(
            "SELECT data_path, CAST(last_import/1000 AS integer) FROM import_records ORDER BY last_import DESC",
        )?;
        let import_records: Vec<crate::types::ImportRecord> = import_stat
            .query_map([], |row| {
                Ok(crate::types::ImportRecord {
                    data_path: row.get(0)?,
                    last_import: row.get(1)?,
                })
            })?
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
}
