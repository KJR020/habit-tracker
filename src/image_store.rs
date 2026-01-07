//! 画像ストレージモジュール

use crate::error::ImageStoreError;
use chrono::{DateTime, Local};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 画像ストレージ
pub struct ImageStore {
    images_dir: PathBuf,
    jpeg_quality: u8,
}

impl ImageStore {
    /// 新しいImageStoreを作成
    pub fn new(images_dir: PathBuf, jpeg_quality: u8) -> Self {
        Self {
            images_dir,
            jpeg_quality,
        }
    }

    /// スクリーンショットをキャプチャし保存
    pub fn capture(&self, timestamp: &DateTime<Local>) -> Result<PathBuf, ImageStoreError> {
        let path = self.get_path(timestamp);

        // 日付ディレクトリを作成
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(ImageStoreError::DirectoryCreationFailed)?;
            }
        }

        // screencaptureコマンドを実行
        // Note: -q オプションは新しいmacOSでは非対応のため、-t jpg のみ使用
        let output = Command::new("screencapture")
            .arg("-x") // サイレント（シャッター音なし）
            .arg("-t")
            .arg("jpg")
            .arg(&path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ImageStoreError::CaptureCommandFailed(format!(
                "screencapture failed: {}",
                stderr
            )));
        }

        Ok(path)
    }

    /// タイムスタンプからファイルパスを生成
    ///
    /// 形式: YYYY-MM-DD/HHMMSS.jpg
    pub fn get_path(&self, timestamp: &DateTime<Local>) -> PathBuf {
        let date_dir = timestamp.format("%Y-%m-%d").to_string();
        let filename = timestamp.format("%H%M%S.jpg").to_string();
        self.images_dir.join(date_dir).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::TempDir;

    #[test]
    fn test_get_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let store = ImageStore::new(temp_dir.path().to_path_buf(), 60);

        let timestamp = Local.with_ymd_and_hms(2024, 12, 30, 10, 30, 45).unwrap();
        let path = store.get_path(&timestamp);

        assert!(path.to_string_lossy().contains("2024-12-30"));
        assert!(path.to_string_lossy().contains("103045.jpg"));
    }

    #[test]
    fn test_get_path_creates_date_directory() {
        let temp_dir = TempDir::new().unwrap();
        let store = ImageStore::new(temp_dir.path().to_path_buf(), 60);

        let timestamp = Local.with_ymd_and_hms(2024, 12, 30, 10, 30, 45).unwrap();
        let path = store.get_path(&timestamp);

        // パスの形式を確認
        let components: Vec<_> = path.components().collect();
        let last_two: Vec<_> = components.iter().rev().take(2).collect();

        assert_eq!(
            last_two[0].as_os_str().to_string_lossy(),
            "103045.jpg"
        );
        assert_eq!(
            last_two[1].as_os_str().to_string_lossy(),
            "2024-12-30"
        );
    }

    #[test]
    fn test_new_image_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = ImageStore::new(temp_dir.path().to_path_buf(), 80);

        assert_eq!(store.jpeg_quality, 80);
        assert_eq!(store.images_dir, temp_dir.path());
    }

    // 注: capture()のテストは実際にスクリーンショットを撮影するため
    // CI環境では実行できない。手動テストまたはE2Eテストで確認する。
}
