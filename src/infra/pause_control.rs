//! 一時停止制御モジュール

use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

/// 一時停止制御
pub struct PauseControl {
    pause_file: PathBuf,
}

impl PauseControl {
    /// 新しいPauseControlを作成
    pub fn new(pause_file: PathBuf) -> Self {
        Self { pause_file }
    }

    /// キャプチャを一時停止
    pub fn pause(&self) -> Result<(), io::Error> {
        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = self.pause_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // 空のフラグファイルを作成
        File::create(&self.pause_file)?;
        Ok(())
    }

    /// キャプチャを再開
    pub fn resume(&self) -> Result<(), io::Error> {
        if self.pause_file.exists() {
            fs::remove_file(&self.pause_file)?;
        }
        Ok(())
    }

    /// 一時停止中かどうかをチェック
    pub fn is_paused(&self) -> bool {
        self.pause_file.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_pause_control() -> (PauseControl, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let pause_file = temp_dir.path().join("pause");
        let control = PauseControl::new(pause_file);
        (control, temp_dir)
    }

    #[test]
    fn test_initial_state_not_paused() {
        let (control, _temp_dir) = create_test_pause_control();
        assert!(!control.is_paused());
    }

    #[test]
    fn test_pause_creates_file() {
        let (control, _temp_dir) = create_test_pause_control();

        assert!(control.pause().is_ok());
        assert!(control.is_paused());
        assert!(control.pause_file.exists());
    }

    #[test]
    fn test_resume_removes_file() {
        let (control, _temp_dir) = create_test_pause_control();

        control.pause().unwrap();
        assert!(control.is_paused());

        control.resume().unwrap();
        assert!(!control.is_paused());
        assert!(!control.pause_file.exists());
    }

    #[test]
    fn test_resume_when_not_paused() {
        let (control, _temp_dir) = create_test_pause_control();

        // 一時停止していない状態でresumeを呼んでもエラーにならない
        assert!(control.resume().is_ok());
        assert!(!control.is_paused());
    }

    #[test]
    fn test_pause_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let pause_file = temp_dir.path().join("subdir").join("pause");
        let control = PauseControl::new(pause_file.clone());

        assert!(control.pause().is_ok());
        assert!(pause_file.exists());
    }

    #[test]
    fn test_double_pause() {
        let (control, _temp_dir) = create_test_pause_control();

        // 2回pauseを呼んでもエラーにならない
        assert!(control.pause().is_ok());
        assert!(control.pause().is_ok());
        assert!(control.is_paused());
    }
}
