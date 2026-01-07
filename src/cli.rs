//! CLIモジュール

use crate::capture::CaptureLoop;
use crate::config::{CliArgs, Config};
use crate::database::Database;
use crate::ocr;
use crate::pause_control::PauseControl;
use crate::report::Report;
use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// Habit Tracker - macOS用作業トラッキングツール
#[derive(Parser, Debug)]
#[command(name = "tracker")]
#[command(about = "macOS用作業トラッキングツール", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// サブコマンド
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// トラッキングを開始
    Start {
        /// キャプチャ間隔（秒）
        #[arg(short, long)]
        interval: Option<u64>,

        /// JPEG品質（0-100）
        #[arg(short, long)]
        quality: Option<u8>,
    },
    /// トラッキングを一時停止
    Pause,
    /// トラッキングを再開
    Resume,
    /// 日次レポートを表示
    Report {
        /// レポート対象日（YYYY-MM-DD形式）
        #[arg(short, long, conflicts_with = "today")]
        date: Option<String>,

        /// 今日のレポートを表示
        #[arg(short, long)]
        today: bool,
    },
    /// 画像からOCRでテキストを抽出
    Ocr {
        /// OCR対象の画像ファイルパス
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// 未処理のキャプチャをOCR処理（件数指定）
        #[arg(short, long)]
        batch: Option<i64>,
    },
}

/// CLIエントリポイント
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { interval, quality } => {
            let cli_args = CliArgs { interval, quality };
            let config = Config::load(&cli_args)?;

            info!("トラッキングを開始します");
            let capture_loop = CaptureLoop::new(config)?;
            capture_loop.setup_signal_handler()?;
            capture_loop.run()?;
        }
        Commands::Pause => {
            let config = Config::load(&CliArgs::default())?;
            let pause_control = PauseControl::new(config.pause_file);
            pause_control.pause()?;
            println!("トラッキングを一時停止しました");
        }
        Commands::Resume => {
            let config = Config::load(&CliArgs::default())?;
            let pause_control = PauseControl::new(config.pause_file);
            pause_control.resume()?;
            println!("トラッキングを再開しました");
        }
        Commands::Report { date, today } => {
            let config = Config::load(&CliArgs::default())?;
            let db = Database::open(&config.db_path)?;
            let report = Report::new(db, config.interval_seconds);

            let target_date = if today {
                Local::now().format("%Y-%m-%d").to_string()
            } else if let Some(d) = date {
                d
            } else {
                Local::now().format("%Y-%m-%d").to_string()
            };

            report.print(&target_date)?;
        }
        Commands::Ocr { file, batch } => {
            if let Some(path) = file {
                // 単一ファイルのOCR
                match ocr::recognize_text(&path) {
                    Ok(text) => {
                        if text.is_empty() {
                            println!("テキストは検出されませんでした");
                        } else {
                            println!("{}", text);
                        }
                    }
                    Err(e) => {
                        eprintln!("OCRエラー: {}", e);
                    }
                }
            } else if let Some(limit) = batch {
                // バッチ処理: 未OCRのキャプチャを処理
                let config = Config::load(&CliArgs::default())?;
                let db = Database::open(&config.db_path)?;
                let captures = db.get_captures_without_ocr(limit)?;

                if captures.is_empty() {
                    println!("OCR未処理のキャプチャはありません");
                } else {
                    println!("{}件のキャプチャをOCR処理します...", captures.len());
                    for capture in captures {
                        if let (Some(id), Some(ref path)) = (capture.id, &capture.image_path) {
                            print!("{} ... ", path);
                            match ocr::recognize_text(&PathBuf::from(path)) {
                                Ok(text) => {
                                    db.update_ocr_text(id, &text)?;
                                    let preview = if text.len() > 50 {
                                        format!("{}...", &text[..50])
                                    } else {
                                        text
                                    };
                                    println!("OK ({})", preview.replace('\n', " "));
                                }
                                Err(e) => {
                                    println!("失敗: {}", e);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("--file または --batch オプションを指定してください");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_command_no_args() {
        let cli = Cli::try_parse_from(["tracker", "start"]);
        assert!(cli.is_ok());

        if let Commands::Start { interval, quality } = cli.unwrap().command {
            assert_eq!(interval, None);
            assert_eq!(quality, None);
        } else {
            panic!("Expected Start command");
        }
    }

    #[test]
    fn test_start_command_with_args() {
        let cli = Cli::try_parse_from(["tracker", "start", "--interval", "30", "--quality", "80"]);
        assert!(cli.is_ok());

        if let Commands::Start { interval, quality } = cli.unwrap().command {
            assert_eq!(interval, Some(30));
            assert_eq!(quality, Some(80));
        } else {
            panic!("Expected Start command");
        }
    }

    #[test]
    fn test_pause_command() {
        let cli = Cli::try_parse_from(["tracker", "pause"]);
        assert!(cli.is_ok());
        assert!(matches!(cli.unwrap().command, Commands::Pause));
    }

    #[test]
    fn test_resume_command() {
        let cli = Cli::try_parse_from(["tracker", "resume"]);
        assert!(cli.is_ok());
        assert!(matches!(cli.unwrap().command, Commands::Resume));
    }

    #[test]
    fn test_report_with_date() {
        let cli = Cli::try_parse_from(["tracker", "report", "--date", "2024-12-30"]);
        assert!(cli.is_ok());

        if let Commands::Report { date, today } = cli.unwrap().command {
            assert_eq!(date, Some("2024-12-30".to_string()));
            assert!(!today);
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_report_with_today() {
        let cli = Cli::try_parse_from(["tracker", "report", "--today"]);
        assert!(cli.is_ok());

        if let Commands::Report { date, today } = cli.unwrap().command {
            assert_eq!(date, None);
            assert!(today);
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_report_date_and_today_conflicts() {
        let cli = Cli::try_parse_from(["tracker", "report", "--date", "2024-12-30", "--today"]);
        assert!(cli.is_err());
    }
}
