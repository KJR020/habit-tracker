//! データベースモジュール

use crate::error::DatabaseError;
use rusqlite::{params, Connection};
use std::path::Path;

/// キャプチャレコードDTO
#[derive(Debug, Clone)]
pub struct CaptureRecord {
    pub id: Option<i64>,
    pub captured_at: String,
    pub image_path: Option<String>,
    pub active_app: String,
    pub window_title: String,
    pub is_paused: bool,
    pub is_private: bool,
    pub ocr_text: Option<String>,
}

/// データベース管理
pub struct Database {
    conn: Connection,
}

impl Database {
    /// データベースを開く（必要に応じて作成）
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        let conn = Connection::open(path)?;

        // WALモードを有効化
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        let db = Database { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// スキーマを初期化
    fn initialize_schema(&self) -> Result<(), DatabaseError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS captures (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                captured_at TEXT NOT NULL,
                image_path TEXT,
                active_app TEXT NOT NULL,
                window_title TEXT NOT NULL DEFAULT '',
                is_paused INTEGER NOT NULL DEFAULT 0,
                is_private INTEGER NOT NULL DEFAULT 0,
                ocr_text TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_captures_captured_at
            ON captures(captured_at);
            "#,
        )?;

        // マイグレーション: ocr_textカラムを追加（既存DBの場合）
        let _ = self
            .conn
            .execute("ALTER TABLE captures ADD COLUMN ocr_text TEXT", []);

        Ok(())
    }

    /// キャプチャレコードを挿入
    pub fn insert_capture(&self, record: &CaptureRecord) -> Result<i64, DatabaseError> {
        self.conn.execute(
            r#"
            INSERT INTO captures (captured_at, image_path, active_app, window_title, is_paused, is_private, ocr_text)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                record.captured_at,
                record.image_path,
                record.active_app,
                record.window_title,
                record.is_paused as i32,
                record.is_private as i32,
                record.ocr_text,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// OCRテキストを更新
    pub fn update_ocr_text(&self, id: i64, ocr_text: &str) -> Result<(), DatabaseError> {
        self.conn.execute(
            "UPDATE captures SET ocr_text = ?1 WHERE id = ?2",
            params![ocr_text, id],
        )?;
        Ok(())
    }

    /// OCRテキストが未設定のキャプチャを取得
    pub fn get_captures_without_ocr(&self, limit: i64) -> Result<Vec<CaptureRecord>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, captured_at, image_path, active_app, window_title, is_paused, is_private, ocr_text
            FROM captures
            WHERE ocr_text IS NULL AND image_path IS NOT NULL
            ORDER BY captured_at DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(CaptureRecord {
                id: Some(row.get(0)?),
                captured_at: row.get(1)?,
                image_path: row.get(2)?,
                active_app: row.get(3)?,
                window_title: row.get(4)?,
                is_paused: row.get::<_, i32>(5)? != 0,
                is_private: row.get::<_, i32>(6)? != 0,
                ocr_text: row.get(7)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// 日付でキャプチャを取得
    pub fn get_captures_by_date(&self, date: &str) -> Result<Vec<CaptureRecord>, DatabaseError> {
        let pattern = format!("{}%", date);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, captured_at, image_path, active_app, window_title, is_paused, is_private, ocr_text
            FROM captures
            WHERE captured_at LIKE ?1
            ORDER BY captured_at ASC
            "#,
        )?;

        let rows = stmt.query_map(params![pattern], |row| {
            Ok(CaptureRecord {
                id: Some(row.get(0)?),
                captured_at: row.get(1)?,
                image_path: row.get(2)?,
                active_app: row.get(3)?,
                window_title: row.get(4)?,
                is_paused: row.get::<_, i32>(5)? != 0,
                is_private: row.get::<_, i32>(6)? != 0,
                ocr_text: row.get(7)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_database_open_creates_schema() {
        let (db, _temp_dir) = create_test_db();

        // テーブルが存在することを確認
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='captures'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_capture() {
        let (db, _temp_dir) = create_test_db();

        let record = CaptureRecord {
            id: None,
            captured_at: "2024-12-30T10:00:00".to_string(),
            image_path: Some("/path/to/image.jpg".to_string()),
            active_app: "VS Code".to_string(),
            window_title: "main.rs".to_string(),
            is_paused: false,
            is_private: false,
            ocr_text: None,
        };

        let id = db.insert_capture(&record).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_captures_by_date() {
        let (db, _temp_dir) = create_test_db();

        // テストデータを挿入
        let records = vec![
            CaptureRecord {
                id: None,
                captured_at: "2024-12-30T10:00:00".to_string(),
                image_path: Some("/path/1.jpg".to_string()),
                active_app: "VS Code".to_string(),
                window_title: "file1.rs".to_string(),
                is_paused: false,
                is_private: false,
                ocr_text: None,
            },
            CaptureRecord {
                id: None,
                captured_at: "2024-12-30T11:00:00".to_string(),
                image_path: Some("/path/2.jpg".to_string()),
                active_app: "Chrome".to_string(),
                window_title: "Google".to_string(),
                is_paused: false,
                is_private: false,
                ocr_text: None,
            },
            CaptureRecord {
                id: None,
                captured_at: "2024-12-31T10:00:00".to_string(),
                image_path: Some("/path/3.jpg".to_string()),
                active_app: "Terminal".to_string(),
                window_title: "".to_string(),
                is_paused: false,
                is_private: false,
                ocr_text: None,
            },
        ];

        for record in &records {
            db.insert_capture(record).unwrap();
        }

        // 2024-12-30のレコードを取得
        let result = db.get_captures_by_date("2024-12-30").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].active_app, "VS Code");
        assert_eq!(result[1].active_app, "Chrome");
    }

    #[test]
    fn test_get_captures_empty_date() {
        let (db, _temp_dir) = create_test_db();

        let result = db.get_captures_by_date("2099-01-01").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_insert_with_null_image_path() {
        let (db, _temp_dir) = create_test_db();

        let record = CaptureRecord {
            id: None,
            captured_at: "2024-12-30T10:00:00".to_string(),
            image_path: None,
            active_app: "VS Code".to_string(),
            window_title: "".to_string(),
            is_paused: true,
            is_private: false,
            ocr_text: None,
        };

        let id = db.insert_capture(&record).unwrap();
        assert!(id > 0);

        let result = db.get_captures_by_date("2024-12-30").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].image_path.is_none());
        assert!(result[0].is_paused);
    }

    #[test]
    fn test_wal_mode_enabled() {
        let (db, _temp_dir) = create_test_db();

        let mode: String = db
            .conn
            .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_index_exists() {
        let (db, _temp_dir) = create_test_db();

        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_captures_captured_at'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
