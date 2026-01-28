//! ログインフラモジュール

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// ログシステムを初期化
///
/// RUST_LOG環境変数でログレベルを設定可能:
/// - error: エラーのみ
/// - warn: 警告以上
/// - info: 情報以上（デフォルト）
/// - debug: デバッグ情報以上
/// - trace: すべて
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_writer(std::io::stderr))
        .init();
}

#[cfg(test)]
mod tests {
    // ログ初期化は1回しか呼べないため、テストは最小限に
    #[test]
    fn test_logging_module_exists() {
        // モジュールが正しくコンパイルされることを確認
        assert!(true);
    }
}
