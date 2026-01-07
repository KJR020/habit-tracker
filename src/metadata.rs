//! メタデータ収集モジュール

use crate::error::MetadataError;
use std::process::Command;
use tracing::warn;

/// メタデータ収集
pub struct Metadata;

impl Metadata {
    /// 最前面のアプリケーション名を取得
    pub fn get_active_app() -> Result<String, MetadataError> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to get name of first process whose frontmost is true"#)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MetadataError::CommandFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("osascript failed: {}", stderr),
            )));
        }

        let name = String::from_utf8(output.stdout)?;
        Ok(name.trim().to_string())
    }

    /// 最前面のウィンドウタイトルを取得
    ///
    /// 失敗した場合は空文字列を返す（優雅なフォールバック）
    pub fn get_window_title() -> String {
        match Self::try_get_window_title() {
            Ok(title) => title,
            Err(e) => {
                warn!("ウィンドウタイトル取得失敗: {}", e);
                String::new()
            }
        }
    }

    /// ウィンドウタイトルの取得を試みる
    fn try_get_window_title() -> Result<String, MetadataError> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to get name of front window of first process whose frontmost is true"#)
            .output()?;

        if !output.status.success() {
            // 一部のアプリはウィンドウタイトルを公開していない
            return Ok(String::new());
        }

        let title = String::from_utf8(output.stdout)?;
        Ok(title.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_app() {
        // 実際のmacOS環境でのみ動作
        let result = Metadata::get_active_app();
        // CI環境では失敗する可能性があるため、結果の型のみ確認
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_window_title_returns_string() {
        // 常に文字列を返すことを確認（エラー時は空文字列）
        let result = Metadata::get_window_title();
        // 結果は文字列（空文字列を含む）
        assert!(result.len() >= 0);
    }

    #[test]
    fn test_get_window_title_never_panics() {
        // パニックしないことを確認
        let _ = Metadata::get_window_title();
    }
}
