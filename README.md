# Habit Tracker MVP

macOS向け個人作業トラッキングツール

## 概要

Habit Trackerは、macOSユーザーの作業時間を自動的に記録し、日次レポートを生成するCLIツールです。定期的にスクリーンショットを撮影し、アクティブなアプリケーションとウィンドウタイトルを記録します。

## 機能

- 定期的なスクリーンショットキャプチャ
- アクティブアプリケーションの自動検出
- ウィンドウタイトルの記録
- 一時停止/再開機能
- 日次活動レポート生成
- アプリ別の時間集計

## インストール

```bash
cargo build --release
```

バイナリは `target/release/tracker` に生成されます。

## 使用方法

### トラッキング開始

```bash
tracker start [OPTIONS]
```

オプション:
- `-i, --interval <秒>` - キャプチャ間隔（デフォルト: 60秒）
- `-q, --quality <0-100>` - JPEG品質（デフォルト: 60）

### 一時停止

```bash
tracker pause
```

### 再開

```bash
tracker resume
```

### レポート表示

```bash
tracker report [OPTIONS]
```

オプション:
- `-d, --date <YYYY-MM-DD>` - 指定日のレポートを表示
- `-t, --today` - 今日のレポートを表示

## 設定

設定ファイル: `~/.habit-tracker/config.toml`

```toml
interval_seconds = 60
jpeg_quality = 60
db_path = "~/.habit-tracker/tracker.db"
images_dir = "~/.habit-tracker/images"
pause_file = "~/.habit-tracker/pause"
```

## データ保存場所

- データベース: `~/.habit-tracker/tracker.db`
- スクリーンショット: `~/.habit-tracker/images/YYYY-MM-DD/HHMMSS.jpg`

## アーキテクチャ

- **config**: 設定管理（TOML + CLI引数）
- **database**: SQLite永続化（WALモード）
- **metadata**: AppleScript経由のアプリ検出
- **image_store**: screencaptureコマンド経由のキャプチャ
- **pause_control**: ファイルベースの一時停止メカニズム
- **capture**: メインキャプチャループとシグナルハンドリング
- **report**: タイムラインとアプリ別時間集計
- **cli**: clapベースのコマンドラインインターフェース

## テスト

```bash
# ユニットテスト
cargo test

# E2Eテスト
./tests/e2e_test.sh
```

## 実装状況

- [x] タスク1-6: コアモジュール（エラー、設定、DB、メタデータ、画像、一時停止）
- [x] タスク7: キャプチャループ
- [x] タスク8: レポート生成
- [x] タスク9: CLIインターフェース
- [x] タスク10: 統合テストとE2E検証

全49個のユニットテストが合格しています。

## ライセンス

MIT