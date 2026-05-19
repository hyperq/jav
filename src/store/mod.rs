use anyhow::Result;
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct HistoryItem {
    pub id: i64,
    pub number: String,
    pub title: String,
    pub link: String,
    pub cover: String,
    pub tags: String,
    pub magnets: String,
    pub viewed_at: String,
    pub favorited: bool,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(path: &str) -> Result<Self> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                number TEXT NOT NULL,
                title TEXT,
                link TEXT,
                cover TEXT DEFAULT '',
                magnets TEXT,
                viewed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                favorited INTEGER DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_number ON history(number);
            CREATE INDEX IF NOT EXISTS idx_fav ON history(favorited);
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        let _ = conn.execute("ALTER TABLE history ADD COLUMN cover TEXT DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE history ADD COLUMN tags TEXT DEFAULT ''", []);
        Ok(Self { conn })
    }

    pub fn add_history(&self, number: &str, title: &str, link: &str, cover: &str, tags: &str, magnets: &str) -> Result<()> {
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM history WHERE number=?1", [number],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            self.conn.execute(
                "UPDATE history SET viewed_at=CURRENT_TIMESTAMP, title=?2, link=?3, cover=CASE WHEN ?4='' THEN cover ELSE ?4 END, tags=CASE WHEN ?5='' THEN tags ELSE ?5 END WHERE id=?1",
                rusqlite::params![id, title, link, cover, tags],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO history (number, title, link, cover, tags, magnets) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![number, title, link, cover, tags, magnets],
            )?;
        }
        Ok(())
    }

    pub fn update_magnets(&self, number: &str, magnets: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE history SET magnets=?2 WHERE number=?1",
            rusqlite::params![number, magnets],
        )?;
        Ok(())
    }

    pub fn get_cached_magnets(&self, number: &str) -> Option<String> {
        self.conn.query_row(
            "SELECT magnets FROM history WHERE number=?1 AND magnets != ''",
            [number], |row| row.get(0),
        ).ok()
    }

    pub fn get_by_number(&self, number: &str) -> Option<HistoryItem> {
        self.conn.query_row(
            "SELECT id,number,title,link,COALESCE(cover,''),COALESCE(tags,''),magnets,COALESCE(viewed_at,''),favorited FROM history WHERE number=?1",
            [number],
            |row| Ok(HistoryItem {
                id: row.get(0)?,
                number: row.get(1)?,
                title: row.get(2)?,
                link: row.get(3)?,
                cover: row.get(4)?,
                tags: row.get(5)?,
                magnets: row.get(6)?,
                viewed_at: row.get(7)?,
                favorited: row.get::<_, i32>(8)? == 1,
            }),
        ).ok()
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE history SET favorited = CASE WHEN favorited=1 THEN 0 ELSE 1 END WHERE id=?1",
            [id],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,number,title,link,COALESCE(cover,''),COALESCE(tags,''),magnets,COALESCE(viewed_at,''),favorited FROM history ORDER BY viewed_at DESC LIMIT ?1",
        )?;
        let items = stmt
            .query_map([limit], |row| {
                Ok(HistoryItem {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    title: row.get(2)?,
                    link: row.get(3)?,
                    cover: row.get(4)?,
                    tags: row.get(5)?,
                    magnets: row.get(6)?,
                    viewed_at: row.get(7)?,
                    favorited: row.get::<_, i32>(8)? == 1,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    pub fn get_favorites(&self, limit: usize) -> Result<Vec<HistoryItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,number,title,link,COALESCE(cover,''),COALESCE(tags,''),magnets,COALESCE(viewed_at,''),favorited FROM history WHERE favorited=1 ORDER BY viewed_at DESC LIMIT ?1",
        )?;
        let items = stmt
            .query_map([limit], |row| {
                Ok(HistoryItem {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    title: row.get(2)?,
                    link: row.get(3)?,
                    cover: row.get(4)?,
                    tags: row.get(5)?,
                    magnets: row.get(6)?,
                    viewed_at: row.get(7)?,
                    favorited: row.get::<_, i32>(8)? == 1,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    pub fn get_config(&self, key: &str, default: &str) -> String {
        self.conn.query_row(
            "SELECT value FROM config WHERE key=?1", [key],
            |row| row.get(0),
        ).unwrap_or_else(|_| default.to_string())
    }

    pub fn set_config(&self, key: &str, value: &str) {
        let _ = self.conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=?2",
            rusqlite::params![key, value],
        );
    }

    pub fn get_config_bool(&self, key: &str, default: bool) -> bool {
        self.get_config(key, if default { "true" } else { "false" }) == "true"
    }

    pub fn set_config_bool(&self, key: &str, value: bool) {
        self.set_config(key, if value { "true" } else { "false" });
    }
}
