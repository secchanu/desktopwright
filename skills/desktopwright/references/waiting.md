# Waiting

タイミング制御と待機の考え方を説明する。

---

## 待機が必要な場面

Windows GUI アプリは非同期に動作するため、操作直後にキャプチャすると意図しない状態が写ることがある。

| 場面 | 典型的な待機時間 |
|---|---|
| ボタンクリック後の画面更新 | 100〜500ms |
| ページ遷移 / ダイアログ表示 | 500〜2000ms |
| ファイル読み込み / 処理完了 | 1000〜5000ms |
| アニメーション完了 | 300〜1000ms |
| ツールチップ表示 | 500〜1500ms |

---

## 待機の方法

### --delay: 固定待機

指定ミリ秒だけ待ってから実行する。シンプルだが過剰・不足の可能性がある。

```bash
# 500ms 待ってからキャプチャ
desktopwright capture --hwnd 132456 --delay 500 --output result.png

# 1000ms 待ってからクリック（クリック直前に 1000ms 待機）
desktopwright click --x 400 --y 300 --coord window --hwnd 132456 --delay 1000

# 500ms 待ってからキー送信
desktopwright key enter --delay 500
```

### --wait-for-diff: 変化検出待機（推奨）

画面に変化が現れるまで最大 N ms 待つ。変化が検出された時点で即座に返る。
Playwright の `waitForSelector` に相当する概念。

```bash
# ボタンをクリックして画面更新を待つ
desktopwright click --x 400 --y 300 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
```

**変化がなかった場合**の扱い:
- 終了コード 0 で正常終了
- 何も stdout に出力されない
- stderr に `タイムアウト: Xms 以内に変化がありませんでした` と出力される

---

## 典型的な操作パターン

### パターン1: クリックして応答を待つ

```bash
# 送信ボタンをクリック（HWND は list コマンドで事前に取得）
desktopwright focus --hwnd 132456
desktopwright click --x 400 --y 500 --coord window --hwnd 132456

# レスポンスが表示されるまで最大 5 秒待つ
desktopwright capture --hwnd 132456 --wait-for-diff 5000 --output response.png
```

### パターン2: テキスト入力してエラーメッセージを確認

```bash
desktopwright focus --hwnd 132456
desktopwright click --x 300 --y 200 --coord window --hwnd 132456
desktopwright type "invalid input"
desktopwright key tab  # 次のフィールドに移動してバリデーション発火

# エラーメッセージが表示されるまで待つ（最大 2 秒）
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output validation.png
```

### パターン3: ページ遷移後のキャプチャ

```bash
desktopwright focus --hwnd 132456
desktopwright key enter  # フォーム送信

# ページが変わったことを確認してからキャプチャ
desktopwright capture --hwnd 132456 --wait-for-diff 5000 --output new-page.png
```

### パターン4: ダイアログの出現を待つ

```bash
desktopwright key ctrl+s  # 保存操作でダイアログが出るかもしれない

# ダイアログが出現するまで待つ
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output dialog.png

# ダイアログがあれば list で確認
desktopwright list --title "保存"
```

---

## 固定待機と変化検出待機の使い分け

| 状況 | 推奨 |
|---|---|
| 画面更新タイミングが不確定 | `--wait-for-diff` |
| 確実に変化することがわかっている | `--wait-for-diff` |
| 変化しないことを確認したい | `--delay` + キャプチャ |
| ツールチップ・ホバーエフェクト | `--delay 500〜1000` |
| アニメーション待ち | `--delay <animation_duration>` |

---

## 差分検出の注意点

### ノイズと閾値

カーソル点滅・アンチエイリアスなど微細な変化を誤検出することがある。
閾値を上げることで無視できる。

```bash
# デフォルト閾値 0.05（5%）
desktopwright capture --target "Chrome" --wait-for-diff 3000

# 閾値を上げてノイズを無視
desktopwright capture --target "Chrome" --wait-for-diff 3000 --diff-threshold 0.1
```

### カーソル点滅があるテキストフィールド

カーソルが点滅するウィンドウでは `--wait-for-diff` が即座に反応してしまう場合がある。
その場合は `--diff-threshold` を上げるか、`--delay` で固定待機を使う。
