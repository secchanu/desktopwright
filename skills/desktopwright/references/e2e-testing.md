# E2E Testing

アプリを自動操作して動作を検証する（AIによるE2Eテスト）のパターンを説明する。

---

## アプリのライフサイクル管理

### アプリを起動する

```bash
# 起動して 1 秒待つ（ウィンドウ出現を待つため）
desktopwright launch "C:\path\to\app.exe" --delay 1000

# 引数を渡す場合
desktopwright launch "C:\path\to\app.exe" "arg1" "arg2"

# 起動直後に wait-for-window で確実に待機する場合（--delay は不要）
desktopwright launch "C:\path\to\app.exe"
desktopwright wait-for-window --process "app" --timeout 10000
```

### ウィンドウが現れるまで待機する

```bash
# プロセス名で待機（起動直後はウィンドウが出ていない場合がある）
desktopwright wait-for-window --process "notepad" --timeout 10000

# タイトルで待機
desktopwright wait-for-window --target "無題" --timeout 5000

# JSON でウィンドウ情報を取得する（HWND を後続コマンドに渡せる）
desktopwright --json wait-for-window --process "app" --timeout 10000
# → { "hwnd": 132456, "title": "...", ... }
```

### アプリを閉じる

```bash
# HWND 直接指定
desktopwright close --hwnd 132456

# タイトルで指定
desktopwright close --target "無題 - メモ帳"

# プロセス名で指定
desktopwright close --process "notepad"
```

---

## タイミング制御

### 操作の間に待機を挟む

```bash
# 固定時間待機（アニメーション完了などを待つ場合）
desktopwright wait 500

# 変化検出付きキャプチャで待機（推奨）
desktopwright capture --hwnd 132456 --wait-for-diff 3000 --output after.png
# → 画面が変化した時点で保存される。タイムアウトした場合は現在の画面を保存。
```

固定時間の `wait` は確実性が低い。可能なかぎり `capture --wait-for-diff` を使うこと。

---

## テキスト値の取得（アサーション）

UIA 経由でUI要素の値を取得する。結果は stdout に出力される。

```bash
# 入力フィールドの現在値を取得（value を返す）
desktopwright get-text --hwnd 132456 --text "Username" --role edit

# ラベルやテキスト要素の内容を取得（name を返す）
desktopwright get-text --hwnd 132456 --text "Status" --role text

# 完全一致モード（部分一致だと複数ヒットする場合に使う）
desktopwright get-text --hwnd 132456 --text "Count: 5" --exact

# タイムアウト指定
desktopwright get-text --hwnd 132456 --text "Result" --timeout 10000
```

### シェルスクリプトでのアサーション例

```bash
result=$(desktopwright get-text --hwnd 132456 --text "Status" --role text)
if [ "$result" = "完了" ]; then
    echo "PASS: ステータスが「完了」になった"
else
    echo "FAIL: 期待値=完了, 実際=$result"
    exit 1
fi
```

---

## UIAが使えないアプリの操作

gpui / DirectX / OpenGL などGPUレンダリングアプリは UIA が効かない場合がある。
この場合は capture + 座標クリックで操作する。

```bash
# 1. 画面をキャプチャして確認
desktopwright capture --hwnd <HWND> --output screen.png

# 2. 画像上の座標からクリック（capture --hwnd の画像座標 = ウィンドウ内座標）
desktopwright focus --hwnd <HWND>
desktopwright click --x 400 --y 300 --coord window --hwnd <HWND>

# 3. 操作後にキャプチャして変化を確認
desktopwright capture --hwnd <HWND> --output after.png
```

### UIA対応状況の確認

```bash
# snapshot を試して出力があれば UIA 対応
desktopwright snapshot --hwnd <HWND>
# → 要素なし または window のみ: UIA 非対応

# ui-tree で詳細確認
desktopwright ui-tree --hwnd <HWND>
```

---

## 典型的なE2Eテストフロー

### メモ帳でファイルを開いて内容を確認する

```bash
# 1. メモ帳を起動
desktopwright launch "notepad.exe" "C:\path\to\file.txt"

# 2. ウィンドウ出現を待機
desktopwright wait-for-window --process "notepad" --timeout 5000

# 3. HWND を確認
desktopwright list

# 4. 内容をキャプチャして確認
desktopwright capture --hwnd 132456 --output content.png

# 5. Edit 要素のテキストを取得
desktopwright get-text --hwnd 132456 --role edit

# 6. メモ帳を閉じる
desktopwright close --hwnd 132456
```

### ダイアログが出ることを確認する

```bash
# 操作を実行
desktopwright click-element --hwnd 132456 --text "削除" --role button

# ダイアログが現れるまで待機（タイトルで待機）
desktopwright wait-for-window --target "確認" --timeout 3000

# ダイアログのHWNDを取得して確認
desktopwright list

# ダイアログを承認
desktopwright dialog-accept --target "確認"
```

---

## UIA が使えないアプリ向けの差分検出テスト

```bash
# ボタンをクリックする前のキャプチャ
desktopwright capture --hwnd <HWND> --output before.png

# ボタンをクリック（座標指定）
desktopwright click --x 200 --y 150 --coord window --hwnd <HWND>

# 変化を待ってキャプチャ（最大 3 秒）
desktopwright capture --hwnd <HWND> --wait-for-diff 3000 --output after.png
# JSON で変化領域も確認
desktopwright --json capture --hwnd <HWND> --wait-for-diff 3000 --output after.png
```
