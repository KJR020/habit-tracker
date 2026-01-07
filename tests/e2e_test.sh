#!/bin/bash
# E2Eワークフローテスト

set -e

# テスト用の一時ディレクトリを作成
TEST_DIR=$(mktemp -d)
export HOME="$TEST_DIR"

echo "=== E2Eテスト開始 ==="
echo "テストディレクトリ: $TEST_DIR"

# バイナリパス
TRACKER="./target/release/tracker"

# 1. Pauseコマンドのテスト
echo ""
echo "[1] Pause/Resume機能のテスト"
$TRACKER pause
if [ ! -f "$TEST_DIR/.habit-tracker/pause" ]; then
    echo "ERROR: pause file not created"
    exit 1
fi
echo "✓ Pauseファイルが作成されました"

$TRACKER resume
if [ -f "$TEST_DIR/.habit-tracker/pause" ]; then
    echo "ERROR: pause file still exists after resume"
    exit 1
fi
echo "✓ Pauseファイルが削除されました"

# 2. Reportコマンドのテスト（データなし）
echo ""
echo "[2] Report機能のテスト（データなし）"
OUTPUT=$($TRACKER report --today 2>&1)
if [[ ! "$OUTPUT" =~ "キャプチャはありませんでした" ]]; then
    echo "ERROR: Expected 'no captures' message"
    echo "Got: $OUTPUT"
    exit 1
fi
echo "✓ データがない場合のレポート出力が正常"

# 3. データベースとディレクトリ構造の検証
echo ""
echo "[3] ディレクトリ構造の検証"
if [ ! -d "$TEST_DIR/.habit-tracker" ]; then
    echo "ERROR: Base directory not created"
    exit 1
fi
echo "✓ ベースディレクトリが作成されました"

if [ ! -f "$TEST_DIR/.habit-tracker/tracker.db" ]; then
    echo "ERROR: Database not created"
    exit 1
fi
echo "✓ データベースファイルが作成されました"

if [ ! -d "$TEST_DIR/.habit-tracker/images" ]; then
    echo "ERROR: Images directory not created"
    exit 1
fi
echo "✓ 画像ディレクトリが作成されました"

# 4. CLIオプションのバリデーション
echo ""
echo "[4] CLI引数のバリデーション"
if $TRACKER report --date "2024-12-30" --today 2>&1 | grep -q "error:"; then
    echo "✓ --dateと--todayの競合が正しく検出されました"
else
    echo "ERROR: Should reject conflicting options"
    exit 1
fi

# クリーンアップ
rm -rf "$TEST_DIR"

echo ""
echo "=== E2Eテスト完了 ==="
echo "すべてのテストが合格しました ✓"
