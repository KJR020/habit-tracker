//! 設定モジュール

use crate::error::ConfigError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct Config {
    /// キャプチャ間隔（秒）
    pub interval_seconds: u64,
    /// JPEG品質（0-100）
    pub jpeg_quality: u8,
    /// データベースファイルパス
    pub db_path: PathBuf,
    /// スクリーンショット保存ディレクトリ
    pub images_dir: PathBuf,
    /// 一時停止フラグファイルパス
    pub pause_file: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let base_dir = home.join(".habit-tracker");

        Self {
            interval_seconds: 60,
            jpeg_quality: 60,
            db_path: base_dir.join("tracker.db"),
            images_dir: base_dir.join("images"),
            pause_file: base_dir.join("pause"),
        }
    }
}

/// TOML設定ファイル用構造体
#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    interval_seconds: Option<u64>,
    jpeg_quality: Option<u8>,
    db_path: Option<String>,
    images_dir: Option<String>,
    pause_file: Option<String>,
}

/// CLI引数
#[derive(Debug, Default)]
pub struct CliArgs {
    pub interval: Option<u64>,
    pub quality: Option<u8>,
}

impl Config {
    /// 設定を読み込む
    ///
    /// 優先順位: CLI引数 > 設定ファイル > デフォルト値
    pub fn load(cli_args: &CliArgs) -> Result<Self, ConfigError> {
        let mut config = Config::default();

        // 設定ファイルを読み込む
        let config_path = config.config_file_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let file_config: FileConfig = toml::from_str(&content)?;
            config.merge_file_config(&file_config);
        }

        // CLI引数で上書き
        config.merge_cli_args(cli_args);

        // バリデーション
        config.validate()?;

        // ディレクトリを作成
        config.ensure_directories()?;

        Ok(config)
    }

    /// 設定ファイルのパスを取得
    fn config_file_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".habit-tracker").join("config.toml")
    }

    /// ファイル設定をマージ
    fn merge_file_config(&mut self, file_config: &FileConfig) {
        if let Some(interval) = file_config.interval_seconds {
            self.interval_seconds = interval;
        }
        if let Some(quality) = file_config.jpeg_quality {
            self.jpeg_quality = quality;
        }
        if let Some(ref path) = file_config.db_path {
            self.db_path = PathBuf::from(path);
        }
        if let Some(ref path) = file_config.images_dir {
            self.images_dir = PathBuf::from(path);
        }
        if let Some(ref path) = file_config.pause_file {
            self.pause_file = PathBuf::from(path);
        }
    }

    /// CLI引数をマージ
    fn merge_cli_args(&mut self, cli_args: &CliArgs) {
        if let Some(interval) = cli_args.interval {
            self.interval_seconds = interval;
        }
        if let Some(quality) = cli_args.quality {
            self.jpeg_quality = quality;
        }
    }

    /// 設定値をバリデート
    fn validate(&self) -> Result<(), ConfigError> {
        if self.interval_seconds == 0 {
            return Err(ConfigError::DirectoryCreationError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "interval_seconds must be greater than 0",
            )));
        }
        if self.jpeg_quality > 100 {
            return Err(ConfigError::DirectoryCreationError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "jpeg_quality must be between 0 and 100",
            )));
        }
        Ok(())
    }

    /// 必要なディレクトリを作成
    fn ensure_directories(&self) -> Result<(), ConfigError> {
        // images_dirを作成
        if !self.images_dir.exists() {
            fs::create_dir_all(&self.images_dir)
                .map_err(ConfigError::DirectoryCreationError)?;
        }

        // db_pathの親ディレクトリを作成
        if let Some(parent) = self.db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(ConfigError::DirectoryCreationError)?;
            }
        }

        // pause_fileの親ディレクトリを作成
        if let Some(parent) = self.pause_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(ConfigError::DirectoryCreationError)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.interval_seconds, 60);
        assert_eq!(config.jpeg_quality, 60);
        assert!(config.db_path.to_string_lossy().contains("tracker.db"));
        assert!(config.images_dir.to_string_lossy().contains("images"));
        assert!(config.pause_file.to_string_lossy().contains("pause"));
    }

    #[test]
    fn test_cli_args_override() {
        let mut config = Config::default();
        let cli_args = CliArgs {
            interval: Some(30),
            quality: Some(80),
        };
        config.merge_cli_args(&cli_args);
        assert_eq!(config.interval_seconds, 30);
        assert_eq!(config.jpeg_quality, 80);
    }

    #[test]
    fn test_file_config_merge() {
        let mut config = Config::default();
        let file_config = FileConfig {
            interval_seconds: Some(120),
            jpeg_quality: Some(90),
            db_path: Some("/tmp/test.db".to_string()),
            images_dir: Some("/tmp/images".to_string()),
            pause_file: Some("/tmp/pause".to_string()),
        };
        config.merge_file_config(&file_config);
        assert_eq!(config.interval_seconds, 120);
        assert_eq!(config.jpeg_quality, 90);
        assert_eq!(config.db_path, PathBuf::from("/tmp/test.db"));
    }

    #[test]
    fn test_cli_overrides_file() {
        let mut config = Config::default();
        let file_config = FileConfig {
            interval_seconds: Some(120),
            jpeg_quality: Some(90),
            ..Default::default()
        };
        config.merge_file_config(&file_config);

        let cli_args = CliArgs {
            interval: Some(30),
            quality: None,
        };
        config.merge_cli_args(&cli_args);

        // CLIが優先
        assert_eq!(config.interval_seconds, 30);
        // ファイル設定が維持
        assert_eq!(config.jpeg_quality, 90);
    }

    #[test]
    fn test_validate_interval_zero() {
        let mut config = Config::default();
        config.interval_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_quality_over_100() {
        let mut config = Config::default();
        config.jpeg_quality = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.images_dir = temp_dir.path().join("images");
        config.db_path = temp_dir.path().join("db").join("tracker.db");
        config.pause_file = temp_dir.path().join("pause");

        assert!(config.ensure_directories().is_ok());
        assert!(config.images_dir.exists());
        assert!(config.db_path.parent().unwrap().exists());
    }

    #[test]
    fn test_load_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.images_dir = temp_dir.path().join("images");
        config.db_path = temp_dir.path().join("tracker.db");
        config.pause_file = temp_dir.path().join("pause");

        assert!(config.validate().is_ok());
        assert!(config.ensure_directories().is_ok());
    }
}
