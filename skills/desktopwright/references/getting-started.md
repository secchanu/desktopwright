# Getting Started

## インストール

GitHub Releases からバイナリをダウンロードして PATH の通ったディレクトリに置く。

```bash
# バージョン確認
desktopwright --version
```

---

## 典型的なワークフロー

### 1. 現在開いているウィンドウを確認する

```bash
desktopwright list
```

出力例:
```
HWND         PID      Process              Title
--------------------------------------------------------------------------------
132456       12345    notepad              無題 - メモ帳
264912       23456    chrome               Google Chrome
```

### 2. 特定のウィンドウをキャプチャする

```bash
# タイトル部分一致でキャプチャ（ファイルに保存）
desktopwright capture --target "メモ帳" --output screen.png

# HWNDで直接指定（最も確実）
desktopwright capture --hwnd 132456 --output screen.png

# プロセス名で指定
desktopwright capture --process notepad --output screen.png

# stdoutに出力（AI向けにパイプで渡す用途）
desktopwright capture --target "メモ帳" > screen.png
```

### 3. snapshot で UI 構造を把握する（任意・推奨）

`snapshot` は playwright の ARIA snapshot に相当する。YAML 形式で要素ツリーを出力し、
各要素に `[ref=eN]` の参照番号を付与する。

```bash
desktopwright snapshot --hwnd 132456
```

出力例:
```yaml
# snapshot: "無題 - メモ帳" (HWND: 132456)
- window "無題 - メモ帳" [ref=e1]:
  - menubar [ref=e2]:
    - menuitem "ファイル(F)" [ref=e3]
    - menuitem "編集(E)" [ref=e4]
  - edit [ref=e5]
  - statusbar [ref=e6]:
    - text "1行, 1列" [ref=e7]
```

ref 番号でクリック:
```bash
desktopwright click-element --hwnd 132456 --ref e3   # "ファイル(F)" メニューをクリック
```

### 4. ウィンドウをフォアグラウンドに移動して操作する

```bash
# テキストでUI要素を直接クリック（座標不要・推奨）
desktopwright click-element --hwnd 132456 --text "保存" --role button

# テキストで Edit 要素にフォーカスしてテキスト入力
desktopwright click-element --hwnd 132456 --text "ファイル名" --role edit
desktopwright type "test.txt"

# フォーカスを当てる（キーボード入力前に必須）
desktopwright focus --hwnd 132456

# 座標でクリック（テキスト指定できない場合）
# capture --hwnd で取得した画像の座標 → --coord window --hwnd でクリック
desktopwright click --x 400 --y 300 --coord window --hwnd 132456

# テキスト入力（日本語も可）
desktopwright type "こんにちは、世界！"

# キー送信
desktopwright key enter
desktopwright key ctrl+s
desktopwright key ctrl+z
```

### 5. 操作後に画面を確認する

```bash
# キャプチャして結果を確認
desktopwright capture --target "メモ帳" --output after.png
```

---

## AI エージェントの典型的なタスク例

### テキストエディタにテキストを入力する

```bash
# 1. HWNDを確認
desktopwright list --process notepad --format json
# → hwnd: 132456

# 2. キャプチャして画面を確認（画像の座標がそのままウィンドウ内座標）
desktopwright capture --hwnd 132456 --output screen.png

# 3. 画像上でテキストエリアの座標(x=400, y=200)を特定してクリック
desktopwright focus --hwnd 132456
desktopwright click --x 400 --y 200 --coord window --hwnd 132456

# 4. テキスト入力
desktopwright type "テスト入力"

# 5. 確認キャプチャ
desktopwright capture --hwnd 132456 --output result.png
```

### ダイアログを操作する

```bash
# ダイアログが開いているか確認
desktopwright list

# フォーカスを当てて Enter キーで確認
desktopwright focus --target "保存の確認"
desktopwright key enter
```

### スクロールする

```bash
# ウィンドウをフォアグラウンドにしてスクロール
desktopwright focus --target "Chrome"
desktopwright scroll --direction down --amount 3
desktopwright scroll --direction down --amount 3

# 特定座標でスクロール
desktopwright scroll --direction up --x 500 --y 400 --amount 5
```

---

## コマンド一覧

| コマンド | 説明 |
|---|---|
| `list` | ウィンドウ一覧表示（クラス名・最小化状態を含む） |
| `capture` | ウィンドウキャプチャ |
| `focus` | ウィンドウをフォアグラウンドに |
| `window` | ウィンドウ状態変更（最小化/最大化/リストア） |
| `resize` | ウィンドウサイズ変更 |
| `snapshot` | アクセシビリティスナップショット（ref番号付き） |
| `click-element` | テキスト/ref でUI要素を検索してクリック |
| `click` | 座標指定でマウスクリック |
| `move` | マウスカーソル移動 |
| `drag` | ドラッグ（mousedown→move→mouseup） |
| `mousedown` | マウスボタンを押したままにする |
| `mouseup` | マウスボタンを離す |
| `scroll` | スクロール |
| `key` | キー送信 |
| `keydown` | キーを押したままにする |
| `keyup` | キーを離す |
| `type` | テキスト入力 |
| `check` | チェックボックスをチェック |
| `uncheck` | チェックボックスのチェックを外す |
| `select` | ドロップダウン・リストの項目を選択 |
| `dialog-accept` | ダイアログを承認（Enter） |
| `dialog-dismiss` | ダイアログを閉じる（Escape） |
| `ui-tree` | UI要素ツリー表示 |
| `foreground` | 現在フォアグラウンドのウィンドウ表示 |

## --json フラグ

全コマンドで `desktopwright --json <コマンド>` の形式で JSON 出力できる。

```bash
# list: hwnd, title, pid, process_name, class_name, visible, minimized, rect を含む
desktopwright --json list

# capture: path, format, width, height（差分検出時は changed_region も含む）
desktopwright --json capture --hwnd 132456 --output screen.png

# click-element: name, role, class_name, rect, click_x, click_y を返す
desktopwright --json click-element --hwnd 132456 --text "OK" --role button

# focus: フォーカスしたウィンドウの WindowInfo を返す
desktopwright --json focus --hwnd 132456
```
