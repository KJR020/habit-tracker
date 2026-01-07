//! レポートモジュール

use crate::database::{CaptureRecord, Database};
use crate::error::ReportError;
use std::collections::HashMap;

/// タイムラインエントリ
#[derive(Debug)]
pub struct TimelineEntry {
    pub time: String,
    pub active_app: String,
    pub window_title: String,
}

/// アプリ別サマリー
#[derive(Debug)]
pub struct AppSummary {
    pub app_name: String,
    pub duration_seconds: u64,
    pub capture_count: u64,
}

/// レポート生成
pub struct Report {
    db: Database,
    interval_seconds: u64,
}

impl Report {
    /// 新しいReportを作成
    pub fn new(db: Database, interval_seconds: u64) -> Self {
        Self {
            db,
            interval_seconds,
        }
    }

    /// タイムラインを生成
    pub fn timeline(&self, date: &str) -> Result<Vec<TimelineEntry>, ReportError> {
        let captures = self.db.get_captures_by_date(date)?;

        let entries: Vec<TimelineEntry> = captures
            .into_iter()
            .map(|c| {
                let time = extract_time(&c.captured_at);
                TimelineEntry {
                    time,
                    active_app: c.active_app,
                    window_title: c.window_title,
                }
            })
            .collect();

        Ok(entries)
    }

    /// アプリ別時間を計算
    pub fn time_by_app(&self, date: &str) -> Result<Vec<AppSummary>, ReportError> {
        let captures = self.db.get_captures_by_date(date)?;

        let mut app_counts: HashMap<String, u64> = HashMap::new();
        for capture in &captures {
            *app_counts.entry(capture.active_app.clone()).or_insert(0) += 1;
        }

        let mut summaries: Vec<AppSummary> = app_counts
            .into_iter()
            .map(|(app_name, count)| AppSummary {
                app_name,
                duration_seconds: count * self.interval_seconds,
                capture_count: count,
            })
            .collect();

        // 時間の降順でソート
        summaries.sort_by(|a, b| b.duration_seconds.cmp(&a.duration_seconds));

        Ok(summaries)
    }

    /// レポートを出力
    pub fn print(&self, date: &str) -> Result<(), ReportError> {
        let timeline = self.timeline(date)?;
        let summaries = self.time_by_app(date)?;

        if timeline.is_empty() {
            println!("{}にキャプチャはありませんでした。", date);
            return Ok(());
        }

        println!("=== {} の活動レポート ===\n", date);

        // タイムライン
        println!("--- タイムライン ---");
        for entry in &timeline {
            let title_display = if entry.window_title.is_empty() {
                String::new()
            } else {
                format!(" - {}", entry.window_title)
            };
            println!("{} | {}{}", entry.time, entry.active_app, title_display);
        }

        println!();

        // アプリ別時間
        println!("--- アプリ別時間 ---");
        for summary in &summaries {
            let duration = format_duration(summary.duration_seconds);
            println!(
                "{}: {} ({} キャプチャ)",
                summary.app_name, duration, summary.capture_count
            );
        }

        Ok(())
    }
}

/// タイムスタンプから時刻部分を抽出
fn extract_time(timestamp: &str) -> String {
    if let Some(time_part) = timestamp.split('T').nth(1) {
        time_part.to_string()
    } else {
        timestamp.to_string()
    }
}

/// 秒を「○時間○分」形式にフォーマット
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}時間{}分", hours, minutes)
    } else {
        format!("{}分", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db_with_data() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir.path().join("test.db")).unwrap();

        // テストデータを挿入
        let records = vec![
            CaptureRecord {
                id: None,
                captured_at: "2024-12-30T10:00:00".to_string(),
                image_path: Some("/path/1.jpg".to_string()),
                active_app: "VS Code".to_string(),
                window_title: "main.rs".to_string(),
                is_paused: false,
                is_private: false,
            },
            CaptureRecord {
                id: None,
                captured_at: "2024-12-30T10:01:00".to_string(),
                image_path: Some("/path/2.jpg".to_string()),
                active_app: "VS Code".to_string(),
                window_title: "lib.rs".to_string(),
                is_paused: false,
                is_private: false,
            },
            CaptureRecord {
                id: None,
                captured_at: "2024-12-30T10:02:00".to_string(),
                image_path: Some("/path/3.jpg".to_string()),
                active_app: "Chrome".to_string(),
                window_title: "Google".to_string(),
                is_paused: false,
                is_private: false,
            },
        ];

        for record in &records {
            db.insert_capture(record).unwrap();
        }

        (db, temp_dir)
    }

    #[test]
    fn test_timeline_generation() {
        let (db, _temp_dir) = create_test_db_with_data();
        let report = Report::new(db, 60);

        let timeline = report.timeline("2024-12-30").unwrap();
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].active_app, "VS Code");
        assert_eq!(timeline[0].time, "10:00:00");
    }

    #[test]
    fn test_time_by_app_calculation() {
        let (db, _temp_dir) = create_test_db_with_data();
        let report = Report::new(db, 60);

        let summaries = report.time_by_app("2024-12-30").unwrap();

        assert_eq!(summaries.len(), 2);

        // VS Codeが最も多い
        assert_eq!(summaries[0].app_name, "VS Code");
        assert_eq!(summaries[0].capture_count, 2);
        assert_eq!(summaries[0].duration_seconds, 120); // 2 * 60

        // Chromeが次
        assert_eq!(summaries[1].app_name, "Chrome");
        assert_eq!(summaries[1].capture_count, 1);
        assert_eq!(summaries[1].duration_seconds, 60);
    }

    #[test]
    fn test_empty_date() {
        let (db, _temp_dir) = create_test_db_with_data();
        let report = Report::new(db, 60);

        let timeline = report.timeline("2099-01-01").unwrap();
        assert!(timeline.is_empty());
    }

    #[test]
    fn test_extract_time() {
        assert_eq!(extract_time("2024-12-30T10:30:45"), "10:30:45");
        assert_eq!(extract_time("invalid"), "invalid");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(60), "1分");
        assert_eq!(format_duration(3600), "1時間0分");
        assert_eq!(format_duration(3660), "1時間1分");
        assert_eq!(format_duration(7260), "2時間1分");
    }
}
