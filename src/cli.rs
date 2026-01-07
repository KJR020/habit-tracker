//! CLIモジュール

use crate::capture::CaptureLoop;
use crate::config::{CliArgs, Config};
use crate::database::Database;
use crate::pause_control::PauseControl;
use crate::report::Report;
use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
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
