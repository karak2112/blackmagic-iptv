use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use iptv_core::{
    Channel, EpgMatch, EpgSummary, GroupInfo, NowNext, PlaylistSummary, Programme, Source,
    SourceType,
};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::AppError;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), AppError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                path_or_url TEXT NOT NULL,
                last_loaded INTEGER
            );

            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                name TEXT NOT NULL,
                group_name TEXT,
                logo_url TEXT,
                stream_url TEXT NOT NULL,
                tvg_id TEXT,
                tvg_name TEXT,
                FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_channels_source ON channels(source_id);
            CREATE INDEX IF NOT EXISTS idx_channels_group ON channels(group_name);
            CREATE INDEX IF NOT EXISTS idx_channels_name ON channels(name);

            CREATE TABLE IF NOT EXISTS programmes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel_epg_id TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                stop_time INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                category TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_programmes_channel_time
                ON programmes(channel_epg_id, start_time);

            CREATE TABLE IF NOT EXISTS epg_matches (
                channel_id TEXT PRIMARY KEY,
                epg_channel_id TEXT NOT NULL,
                match_method TEXT NOT NULL,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS favorites (
                channel_id TEXT PRIMARY KEY,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn upsert_source(&self, source: &Source) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO sources (id, name, source_type, path_or_url, last_loaded)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               name=excluded.name,
               source_type=excluded.source_type,
               path_or_url=excluded.path_or_url,
               last_loaded=excluded.last_loaded",
            params![
                source.id,
                source.name,
                source_type_str(&source.source_type),
                source.path_or_url,
                source.last_loaded.map(|t| t.timestamp()),
            ],
        )?;
        Ok(())
    }

    pub fn replace_channels(&self, source_id: &str, channels: &[Channel]) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM channels WHERE source_id = ?1", [source_id])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO channels (id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for ch in channels {
                stmt.execute(params![
                    ch.id,
                    ch.source_id,
                    ch.name,
                    ch.group,
                    ch.logo_url,
                    ch.stream_url,
                    ch.tvg_id,
                    ch.tvg_name,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_programmes(&self, programmes: &[Programme]) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM programmes", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO programmes (channel_epg_id, start_time, stop_time, title, description, category)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for p in programmes {
                stmt.execute(params![
                    p.channel_epg_id,
                    p.start.timestamp(),
                    p.stop.timestamp(),
                    p.title,
                    p.description,
                    p.category,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_epg_matches(&self, matches: &[EpgMatch]) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM epg_matches", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO epg_matches (channel_id, epg_channel_id, match_method)
                 VALUES (?1, ?2, ?3)",
            )?;
            for m in matches {
                stmt.execute(params![m.channel_id, m.epg_channel_id, m.method.as_str()])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_sources(&self) -> Result<Vec<Source>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, source_type, path_or_url, last_loaded FROM sources ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Source {
                id: row.get(0)?,
                name: row.get(1)?,
                source_type: parse_source_type(row.get::<_, String>(2)?),
                path_or_url: row.get(3)?,
                last_loaded: row
                    .get::<_, Option<i64>>(4)?
                    .and_then(|ts| DateTime::from_timestamp(ts, 0)),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn list_groups(&self, source_id: Option<&str>) -> Result<Vec<GroupInfo>, AppError> {
        let map_row = |row: &rusqlite::Row<'_>| {
            Ok(GroupInfo {
                name: row.get(0)?,
                channel_count: row.get(1)?,
            })
        };

        if let Some(id) = source_id {
            let mut stmt = self.conn.prepare(
                "SELECT COALESCE(group_name, 'Uncategorized') AS g, COUNT(*) AS c
                 FROM channels WHERE source_id = ?1 GROUP BY g ORDER BY g",
            )?;
            let rows = stmt.query_map([id], map_row)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT COALESCE(group_name, 'Uncategorized') AS g, COUNT(*) AS c
                 FROM channels GROUP BY g ORDER BY g",
            )?;
            let rows = stmt.query_map([], map_row)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        }
    }

    pub fn list_channels(
        &self,
        source_id: Option<&str>,
        group: Option<&str>,
        search: Option<&str>,
        favorites_only: bool,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Channel>, usize), AppError> {
        let search_pattern = search
            .filter(|s| !s.trim().is_empty())
            .map(|q| format!("%{}%", q.trim()));

        let mut conditions = Vec::<String>::new();
        let mut bind: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let from_clause = if favorites_only {
            "FROM channels INNER JOIN favorites ON favorites.channel_id = channels.id"
        } else {
            "FROM channels"
        };

        if let Some(sid) = source_id {
            conditions.push("channels.source_id = ?".into());
            bind.push(Box::new(sid.to_string()));
        }
        if let Some(g) = group {
            if g == "Uncategorized" {
                conditions.push("channels.group_name IS NULL".into());
            } else {
                conditions.push("channels.group_name = ?".into());
                bind.push(Box::new(g.to_string()));
            }
        }
        if let Some(ref pattern) = search_pattern {
            conditions.push("(channels.name LIKE ? OR channels.tvg_name LIKE ?)".into());
            bind.push(Box::new(pattern.clone()));
            bind.push(Box::new(pattern.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) {from_clause} {where_clause}");
        let bind_refs: Vec<&dyn rusqlite::ToSql> = bind.iter().map(|b| b.as_ref()).collect();
        let total: usize = self
            .conn
            .query_row(count_sql.as_str(), bind_refs.as_slice(), |row| row.get(0))?;

        let list_sql = format!(
            "SELECT channels.id, channels.source_id, channels.name, channels.group_name, channels.logo_url, channels.stream_url, channels.tvg_id, channels.tvg_name
             {from_clause} {where_clause} ORDER BY channels.name LIMIT ? OFFSET ?"
        );
        let mut list_bind = bind;
        list_bind.push(Box::new(limit as i64));
        list_bind.push(Box::new(offset as i64));
        let list_refs: Vec<&dyn rusqlite::ToSql> = list_bind.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&list_sql)?;
        let rows = stmt.query_map(list_refs.as_slice(), |row| {
            Ok(Channel {
                id: row.get(0)?,
                source_id: row.get(1)?,
                name: row.get(2)?,
                group: row.get(3)?,
                logo_url: row.get(4)?,
                stream_url: row.get(5)?,
                tvg_id: row.get(6)?,
                tvg_name: row.get(7)?,
            })
        })?;

        Ok((rows.collect::<Result<Vec<_>, _>>()?, total))
    }

    pub fn get_channel(&self, channel_id: &str) -> Result<Option<Channel>, AppError> {
        self.conn
            .query_row(
                "SELECT id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name
                 FROM channels WHERE id = ?1",
                [channel_id],
                |row| {
                    Ok(Channel {
                        id: row.get(0)?,
                        source_id: row.get(1)?,
                        name: row.get(2)?,
                        group: row.get(3)?,
                        logo_url: row.get(4)?,
                        stream_url: row.get(5)?,
                        tvg_id: row.get(6)?,
                        tvg_name: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(AppError::from)
    }

    /// Next or previous channel in playlist order (by name), wrapping at ends.
    pub fn adjacent_channel(&self, channel_id: &str, delta: i32) -> Result<Option<Channel>, AppError> {
        let current = match self.get_channel(channel_id)? {
            Some(c) => c,
            None => return Ok(None),
        };

        let neighbor = if delta >= 0 {
            self.query_neighbor_after(&current)?
                .or_else(|| self.query_first_channel(&current.source_id))
        } else {
            self.query_neighbor_before(&current)?
                .or_else(|| self.query_last_channel(&current.source_id))
        };

        Ok(neighbor.filter(|c| c.id != channel_id))
    }

    fn query_neighbor_after(&self, current: &Channel) -> Result<Option<Channel>, AppError> {
        self.conn
            .query_row(
                "SELECT id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name
                 FROM channels
                 WHERE source_id = ?1 AND name > ?2
                 ORDER BY name ASC LIMIT 1",
                rusqlite::params![current.source_id, current.name],
                Self::map_channel_row,
            )
            .optional()
            .map_err(AppError::from)
    }

    fn query_neighbor_before(&self, current: &Channel) -> Result<Option<Channel>, AppError> {
        self.conn
            .query_row(
                "SELECT id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name
                 FROM channels
                 WHERE source_id = ?1 AND name < ?2
                 ORDER BY name DESC LIMIT 1",
                rusqlite::params![current.source_id, current.name],
                Self::map_channel_row,
            )
            .optional()
            .map_err(AppError::from)
    }

    fn query_first_channel(&self, source_id: &str) -> Option<Channel> {
        self.conn
            .query_row(
                "SELECT id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name
                 FROM channels WHERE source_id = ?1 ORDER BY name ASC LIMIT 1",
                [source_id],
                Self::map_channel_row,
            )
            .optional()
            .ok()
            .flatten()
    }

    fn query_last_channel(&self, source_id: &str) -> Option<Channel> {
        self.conn
            .query_row(
                "SELECT id, source_id, name, group_name, logo_url, stream_url, tvg_id, tvg_name
                 FROM channels WHERE source_id = ?1 ORDER BY name DESC LIMIT 1",
                [source_id],
                Self::map_channel_row,
            )
            .optional()
            .ok()
            .flatten()
    }

    fn map_channel_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Channel> {
        Ok(Channel {
            id: row.get(0)?,
            source_id: row.get(1)?,
            name: row.get(2)?,
            group: row.get(3)?,
            logo_url: row.get(4)?,
            stream_url: row.get(5)?,
            tvg_id: row.get(6)?,
            tvg_name: row.get(7)?,
        })
    }

    pub fn get_epg_channel_id(&self, channel_id: &str) -> Result<Option<String>, AppError> {
        self.conn
            .query_row(
                "SELECT epg_channel_id FROM epg_matches WHERE channel_id = ?1",
                [channel_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn now_next_for_epg_ids(&self, epg_ids: &[String]) -> Result<Vec<(String, NowNext)>, AppError> {
        if epg_ids.is_empty() {
            return Ok(Vec::new());
        }

        let now = Utc::now().timestamp();
        let placeholders = epg_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT channel_epg_id, start_time, stop_time, title, description, category
             FROM programmes
             WHERE channel_epg_id IN ({placeholders})
               AND stop_time > ? 
             ORDER BY channel_epg_id, start_time"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = epg_ids
            .iter()
            .map(|id| Box::new(id.clone()) as Box<dyn rusqlite::ToSql>)
            .collect();
        params_vec.push(Box::new(now));

        let mut rows = stmt.query(rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())))?;

        let mut result_map: std::collections::HashMap<String, NowNext> = std::collections::HashMap::new();

        while let Some(row) = rows.next()? {
            let channel_epg_id: String = row.get(0)?;
            let start: i64 = row.get(1)?;
            let stop: i64 = row.get(2)?;
            let title: String = row.get(3)?;
            let description: Option<String> = row.get(4)?;
            let category: Option<String> = row.get(5)?;

            let programme = Programme {
                channel_epg_id: channel_epg_id.clone(),
                start: DateTime::from_timestamp(start, 0).unwrap_or_else(Utc::now),
                stop: DateTime::from_timestamp(stop, 0).unwrap_or_else(Utc::now),
                title,
                description,
                category,
            };

            let entry = result_map.entry(channel_epg_id).or_default();
            let prog_start = programme.start.timestamp();
            let prog_stop = programme.stop.timestamp();

            if prog_start <= now && prog_stop > now {
                entry.now = Some(programme);
            } else if prog_start > now && entry.next.is_none() {
                entry.next = Some(programme);
            }
        }

        Ok(epg_ids
            .iter()
            .map(|id| (id.clone(), result_map.remove(id).unwrap_or_default()))
            .collect())
    }

    pub fn playlist_summary(&self, source_id: &str) -> Result<PlaylistSummary, AppError> {
        let channel_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM channels WHERE source_id = ?1",
            [source_id],
            |row| row.get(0),
        )?;
        let group_count: usize = self.conn.query_row(
            "SELECT COUNT(DISTINCT COALESCE(group_name, '')) FROM channels WHERE source_id = ?1",
            [source_id],
            |row| row.get(0),
        )?;
        Ok(PlaylistSummary {
            source_id: source_id.to_string(),
            channel_count,
            group_count,
        })
    }

    pub fn epg_summary(&self) -> Result<EpgSummary, AppError> {
        let programme_count: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM programmes", [], |row| row.get(0))?;
        let channel_count: usize = self.conn.query_row(
            "SELECT COUNT(DISTINCT channel_epg_id) FROM programmes",
            [],
            |row| row.get(0),
        )?;
        let matched_count: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM epg_matches", [], |row| row.get(0))?;
        Ok(EpgSummary {
            channel_count,
            programme_count,
            matched_count,
        })
    }

    pub fn all_channels(&self) -> Result<Vec<Channel>, AppError> {
        let (channels, _) = self.list_channels(None, None, None, false, 0, usize::MAX)?;
        Ok(channels)
    }

    pub fn toggle_favorite(&self, channel_id: &str) -> Result<bool, AppError> {
        let exists: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM favorites WHERE channel_id = ?1",
                [channel_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_some() {
            self.conn
                .execute("DELETE FROM favorites WHERE channel_id = ?1", [channel_id])?;
            Ok(false)
        } else {
            self.conn
                .execute("INSERT INTO favorites (channel_id) VALUES (?1)", [channel_id])?;
            Ok(true)
        }
    }

    pub fn list_favorite_ids(&self) -> Result<Vec<String>, AppError> {
        let mut stmt = self.conn.prepare("SELECT channel_id FROM favorites")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(AppError::from)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

pub fn new_source_id() -> String {
    Uuid::new_v4().to_string()
}

fn source_type_str(t: &SourceType) -> &'static str {
    match t {
        SourceType::M3uLocal => "m3u_local",
        SourceType::M3uRemote => "m3u_remote",
        SourceType::XmltvLocal => "xmltv_local",
        SourceType::XmltvRemote => "xmltv_remote",
    }
}

fn parse_source_type(raw: String) -> SourceType {
    match raw.as_str() {
        "m3u_remote" => SourceType::M3uRemote,
        "xmltv_local" => SourceType::XmltvLocal,
        "xmltv_remote" => SourceType::XmltvRemote,
        _ => SourceType::M3uLocal,
    }
}

pub fn cache_dir(base: &Path) -> PathBuf {
    base.join("cache")
}

#[cfg(test)]
mod tests;
