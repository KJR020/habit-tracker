//! キャプチャループモジュール

use crate::config::Config;
use crate::database::{CaptureRecord, Database};
use crate::error::CaptureError;
use crate::image_store::ImageStore;
use crate::metadata::Metadata;
use crate::ocr;
use crate::pause_control::PauseControl;

use chrono::Local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

/// キャプチャループ
pub struct CaptureLoop {
    config: Config,
    db: Database,
    image_store: ImageStore,
    pause_control: PauseControl,
    running: Arc<AtomicBool>,
}

impl CaptureLoop {
    /// 新しいCaptureLoopを作成
    pub fn new(config: Config) -> Result<Self, CaptureError> {
        let db = Database::open(&config.db_path)?;
        let image_store = ImageStore::new(config.images_dir.clone(), config.jpeg_quality);
        let pause_control = PauseControl::new(config.pause_file.clone());
        let running = Arc::new(AtomicBool::new(true));

        Ok(Self {
            config,
            db,
            image_store,
            pause_control,
            running,
        })
    }

    /// シグナルハンドラーをセットアップ
    pub fn setup_signal_handler(&self) -> Result<(), CaptureError> {
        let running = Arc::clone(&self.running);

        ctrlc::set_handler(move || {
            info!("シャットダウンシグナルを受信しました");
            running.store(false, Ordering::SeqCst);
        })
        .map_err(|e| CaptureError::SignalHandlerError(e.to_string()))?;

        Ok(())
    }

    /// キャプチャループを実行
    pub fn run(&self) -> Result<(), CaptureError> {
        info!(
            "キャプチャループを開始します（間隔: {}秒）",
            self.config.interval_seconds
        );

        while self.running.load(Ordering::SeqCst) {
            // 一時停止チェック
            if self.pause_control.is_paused() {
                info!("一時停止中...");
                thread::sleep(Duration::from_secs(self.config.interval_seconds));
                continue;
            }

            // キャプチャサイクルを実行
            if let Err(e) = self.capture_cycle() {
                error!("キャプチャサイクルでエラー: {}", e);
                // エラーが発生してもループは継続
            }

            // インターバル待機
            thread::sleep(Duration::from_secs(self.config.interval_seconds));
        }

        info!("キャプチャループを終了します");
        Ok(())
    }

    /// 単一のキャプチャサイクル
    fn capture_cycle(&self) -> Result<(), CaptureError> {
        let timestamp = Local::now();

        // メタデータを収集
        let active_app = match Metadata::get_active_app() {
            Ok(app) => app,
            Err(e) => {
                warn!("アクティブアプリ取得失敗: {}", e);
                "Unknown".to_string()
            }
        };
        let window_title = Metadata::get_window_title();

        // スクリーンショットをキャプチャ
        let image_path = match self.image_store.capture(&timestamp) {
            Ok(path) => Some(path),
            Err(e) => {
                warn!("スクリーンショットキャプチャ失敗: {}", e);
                None
            }
        };

        // OCRでテキストを抽出
        let ocr_text = if let Some(ref path) = image_path {
            match ocr::recognize_text(path) {
                Ok(text) => {
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                }
                Err(e) => {
                    warn!("OCR失敗: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // データベースに記録
        let record = CaptureRecord {
            id: None,
            captured_at: timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
            image_path: image_path.map(|p| p.to_string_lossy().to_string()),
            active_app,
            window_title,
            is_paused: false,
            is_private: false,
            ocr_text,
        };

        self.db.insert_capture(&record)?;
        info!("キャプチャ完了: {}", record.captured_at);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            interval_seconds: 1,
            jpeg_quality: 60,
            db_path: temp_dir.path().join("test.db"),
            images_dir: temp_dir.path().join("images"),
            pause_file: temp_dir.path().join("pause"),
        };
        (config, temp_dir)
    }

    #[test]
    fn test_capture_loop_creation() {
        let (config, _temp_dir) = create_test_config();
        let result = CaptureLoop::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_running_flag_initial_state() {
        let (config, _temp_dir) = create_test_config();
        let loop_ = CaptureLoop::new(config).unwrap();
        assert!(loop_.running.load(Ordering::SeqCst));
    }

    #[test]
    fn test_running_flag_can_be_stopped() {
        let (config, _temp_dir) = create_test_config();
        let loop_ = CaptureLoop::new(config).unwrap();

        loop_.running.store(false, Ordering::SeqCst);
        assert!(!loop_.running.load(Ordering::SeqCst));
    }
}
