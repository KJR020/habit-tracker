//! エラー型定義モジュール

use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// 設定エラー
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IOエラー: {0}")]
    IoError(#[from] io::Error),

    #[error("TOML解析エラー: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("ディレクトリ作成エラー: {0}")]
    DirectoryCreationError(io::Error),
}

/// データベースエラー
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLiteエラー: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("IOエラー: {0}")]
    IoError(#[from] io::Error),

    #[error("マイグレーションエラー: {0}")]
    MigrationError(String),
}

/// メタデータエラー
#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("コマンド実行失敗: {0}")]
    CommandFailed(#[from] io::Error),

    #[error("UTF-8変換エラー: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

/// 画像ストレージエラー
#[derive(Error, Debug)]
pub enum ImageStoreError {
    #[error("コマンド実行失敗: {0}")]
    CommandFailed(#[from] io::Error),

    #[error("ディレクトリ作成失敗: {0}")]
    DirectoryCreationFailed(io::Error),

    #[error("キャプチャコマンド失敗: {0}")]
    CaptureCommandFailed(String),
}

/// キャプチャエラー
#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("データベースエラー: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("設定エラー: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("初期化エラー: {0}")]
    InitializationError(String),

    #[error("シグナルハンドラーエラー: {0}")]
    SignalHandlerError(String),
}

/// レポートエラー
#[derive(Error, Debug)]
pub enum ReportError {
    #[error("データベースエラー: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("無効な日付: {0}")]
    InvalidDate(String),
}

/// OCRエラー
#[derive(Error, Debug)]
pub enum OcrError {
    #[error("画像が見つかりません: {0}")]
    ImageNotFound(String),

    #[error("OCR実行失敗: {0}")]
    ExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::DirectoryCreationError(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
        assert!(err.to_string().contains("ディレクトリ作成エラー"));
    }

    #[test]
    fn test_database_error_display() {
        let err = DatabaseError::MigrationError("テストエラー".to_string());
        assert!(err.to_string().contains("マイグレーションエラー"));
    }

    #[test]
    fn test_metadata_error_display() {
        let err = MetadataError::CommandFailed(io::Error::new(
            io::ErrorKind::NotFound,
            "command not found",
        ));
        assert!(err.to_string().contains("コマンド実行失敗"));
    }

    #[test]
    fn test_image_store_error_display() {
        let err = ImageStoreError::CaptureCommandFailed("screencapture failed".to_string());
        assert!(err.to_string().contains("キャプチャコマンド失敗"));
    }

    #[test]
    fn test_capture_error_display() {
        let err = CaptureError::InitializationError("初期化に失敗".to_string());
        assert!(err.to_string().contains("初期化エラー"));
    }

    #[test]
    fn test_report_error_display() {
        let err = ReportError::InvalidDate("2024-13-45".to_string());
        assert!(err.to_string().contains("無効な日付"));
    }
}
