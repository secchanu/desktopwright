# Input

マウス・キーボード操作のパターンを説明する。

---

## 前提: フォーカスを当ててから操作する

キーボード操作（`key`、`type`）はフォアグラウンドウィンドウに送られる。
必ず事前に `focus` を実行すること。

```bash
desktopwright focus --target "メモ帳"
desktopwright type "Hello"  # メモ帳に入力される
```

---

## マウスクリック

### 推奨: --coord window でキャプチャ画像の座標をそのまま使う

`capture --hwnd <n>` で取得した画像の座標系は**ウィンドウ内座標**（ウィンドウ左上を原点）。
`--coord window --hwnd <n>` を使えば、画像上の座標をそのままクリックに使える。

```bash
# 1. HWNDを確認
desktopwright list --process notepad --format json
# → hwnd: 132456

# 2. キャプチャして画像上の座標(x=400, y=200)を特定
desktopwright capture --hwnd 132456 --output screen.png

# 3. 画像の座標をそのままウィンドウ座標としてクリック（変換不要）
desktopwright click --x 400 --y 200 --coord window --hwnd 132456
```

**注意**: `capture --hwnd n` の画像座標をスクリーン座標として `click --x 400 --y 200`（`--coord screen`）に使うと、ウィンドウ位置分だけズレる。必ず `--coord window --hwnd <n>` を使うこと。

### スクリーン絶対座標（デスクトップ全体操作時）

ウィンドウに紐付かないデスクトップ上の座標を指定したい場合のみ使う。

```bash
desktopwright click --x 1920 --y 540 --coord screen
```

### その他のクリック種別

```bash
# 右クリック
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --button right

# 中クリック
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --button middle

# ダブルクリック
desktopwright click --x 400 --y 200 --coord window --hwnd 132456 --double
```

### クリック前の待機

```bash
desktopwright click --x 400 --y 300 --coord window --hwnd 132456 --delay 500
```

---

## マウスカーソル移動

クリックせずにカーソルだけ移動する。ホバーエフェクトやツールチップを確認する場合に使う。

```bash
desktopwright move --x 400 --y 300

# ウィンドウ相対座標で移動
desktopwright move --x 100 --y 50 --coord window --hwnd 132456
```

---

## スクロール

```bash
# 下にスクロール（デフォルト 3 ノッチ）
desktopwright scroll --direction down

# 上にスクロール（5 ノッチ）
desktopwright scroll --direction up --amount 5

# 左右スクロール
desktopwright scroll --direction left --amount 3
desktopwright scroll --direction right --amount 3

# 特定座標でスクロール
desktopwright scroll --direction down --x 500 --y 400 --amount 3
```

---

## キー送信

### 基本キー

```bash
desktopwright key enter
desktopwright key escape
desktopwright key tab
desktopwright key space
desktopwright key backspace
desktopwright key delete
```

### ファンクションキー

```bash
desktopwright key f1
desktopwright key f5   # ブラウザのリロード等
desktopwright key f12  # 開発者ツール等
```

### カーソルキー

```bash
desktopwright key up
desktopwright key down
desktopwright key left
desktopwright key right
desktopwright key home
desktopwright key end
desktopwright key pageup
desktopwright key pagedown
```

### 修飾キーの組み合わせ

```bash
# コピー・ペースト
desktopwright key ctrl+c
desktopwright key ctrl+v
desktopwright key ctrl+x

# 元に戻す・やり直し
desktopwright key ctrl+z
desktopwright key ctrl+y

# 保存
desktopwright key ctrl+s

# 全選択
desktopwright key ctrl+a

# 検索
desktopwright key ctrl+f

# ウィンドウを閉じる
desktopwright key alt+f4

# タスクマネージャー
desktopwright key ctrl+shift+escape

# 複数修飾キー
desktopwright key ctrl+shift+t   # ブラウザのタブを元に戻す
```

### キー入力前の待機

```bash
desktopwright key enter --delay 500
```

---

## テキスト入力

テキスト入力はクリップボード貼り付け方式を使う。日本語・特殊文字も問題なく入力できる。

```bash
# ASCII テキスト
desktopwright type "Hello, World!"

# 日本語
desktopwright type "こんにちは、世界！"

# 特殊文字
desktopwright type "Price: $100 (tax included)"

# 改行を含む場合は Enter キーを別途送る
desktopwright type "Line 1"
desktopwright key enter
desktopwright type "Line 2"
```

### 注意: クリップボードの上書き

`type` コマンドは実行中にクリップボードの内容を一時的に書き換える。
既存のクリップボード内容が消えることに注意。

---

## ドラッグ

### 座標指定でドラッグ

```bash
# ウィンドウ内座標でドラッグ
desktopwright drag --from-x 100 --from-y 200 --to-x 300 --to-y 400 --coord window --hwnd 132456

# スクリーン絶対座標でドラッグ
desktopwright drag --from-x 500 --from-y 300 --to-x 800 --to-y 500

# 右クリックドラッグ
desktopwright drag --from-x 100 --from-y 200 --to-x 300 --to-y 400 --button right --hwnd 132456 --coord window

# 移動ステップ数を増やして滑らかにする（デフォルト: 10）
desktopwright drag --from-x 100 --from-y 200 --to-x 300 --to-y 400 --steps 30 --hwnd 132456 --coord window
```

### UI 要素間でドラッグ（UIA でアクセシブル名指定）

```bash
# アイテムを別の場所にドラッグ
desktopwright drag --from-element "ドラッグ元" --to-element "ドロップ先" --hwnd 132456
```

### mousedown / mouseup を個別に使う（修飾キーとの組み合わせ）

```bash
# Shift+ドラッグで範囲選択する例
desktopwright mousedown --x 100 --y 200 --coord window --hwnd 132456
desktopwright keydown shift
desktopwright move --x 300 --y 200 --coord window --hwnd 132456
desktopwright mouseup --x 300 --y 200 --coord window --hwnd 132456
desktopwright keyup shift
```

---

## keydown / keyup（修飾キーの保持）

修飾キーを押したままにして、その間に別の操作を行う場合に使用する。

```bash
# Ctrl を押しながら複数クリック（複数選択）
desktopwright keydown ctrl
desktopwright click --x 100 --y 200 --coord window --hwnd 132456
desktopwright click --x 100 --y 250 --coord window --hwnd 132456
desktopwright keyup ctrl
```

---

## チェックボックス操作（check / uncheck）

UIA の TogglePattern を使ってチェック状態を確実に設定する。

```bash
# チェックを入れる
desktopwright check --text "同意する" --hwnd 132456

# チェックを外す
desktopwright uncheck --text "同意する" --hwnd 132456
```

---

## ドロップダウン選択（select）

UIA の SelectionItemPattern を使ってリスト・コンボボックスの項目を選択する。

```bash
# コンボボックスのラベルを指定して項目を選択
desktopwright select --value "日本語" --element "言語" --hwnd 132456

# ラベルなしで項目を直接選択（既に展開されている場合）
desktopwright select --value "オプション A" --hwnd 132456
```

---

## ダイアログ操作（dialog-accept / dialog-dismiss）

```bash
# ダイアログを承認（Enter キーを送信）
desktopwright dialog-accept

# ターゲット指定でダイアログを承認
desktopwright dialog-accept --target "保存の確認"

# ダイアログを閉じる（Escape キーを送信）
desktopwright dialog-dismiss
```

---

## UI 要素のクリック（ui-tree を活用）

`ui-tree` で取得した要素の `rect` もウィンドウ内座標系。そのまま `--coord window` で使える。

```bash
# UI 要素ツリーを確認
desktopwright ui-tree --hwnd 132456

# 出力例:
# └── Window "無題 - メモ帳" [0,0 800x600]
#     ├── Edit "" id=15 [8,55 784x537]
#     └── TitleBar "" [0,0 800x30]

# Edit 要素の中央をクリック: x = 8 + 784/2 = 400, y = 55 + 537/2 = 324
desktopwright click --x 400 --y 324 --coord window --hwnd 132456
```
