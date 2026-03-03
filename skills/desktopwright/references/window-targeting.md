# Window Targeting

ウィンドウの特定方法と、HWND・タイトル・プロセス名の使い分けについて説明する。

---

## ターゲット指定の3つの方法

### 1. HWND直接指定（最も確実）

HWND はセッション中にウィンドウを一意に識別するハンドル。
`list` コマンドで確認できる。

```bash
# 十進数
desktopwright capture --hwnd 132456

# 十六進数（0x prefix）
desktopwright capture --hwnd 0x205B0
```

**使うべき場面**: 同名タイトル・同名プロセスが複数ある場合、または繰り返し操作する場合。

### 2. タイトル部分一致

ウィンドウタイトルの一部を指定する（大文字・小文字を区別しない）。

```bash
desktopwright focus --target "メモ帳"
desktopwright focus --target "Notepad"   # 英語版でも同じ
desktopwright focus --target "無題 - "   # 前方一致的に使える
```

**注意**: 複数のウィンドウが一致するとエラーになる。HWNDで再指定すること。

### 3. プロセス名部分一致

プロセスの実行ファイル名（拡張子なし）を指定する。

```bash
desktopwright focus --process notepad
desktopwright focus --process chrome
desktopwright focus --process "Microsoft Visual Studio"
```

### 4. タイトル + プロセス名の組み合わせ

両方指定することで絞り込みの精度が上がる。

```bash
desktopwright focus --target "設定" --process SystemSettings
```

---

## list コマンドの活用

### 全ウィンドウを確認

```bash
desktopwright list
```

### フィルタリング

```bash
# プロセス名でフィルタ
desktopwright list --process notepad

# タイトルでフィルタ
desktopwright list --title "Chrome"

# JSON形式で出力（スクリプト処理に便利）
desktopwright list --format json
```

JSON出力の構造:
```json
[
  {
    "hwnd": 132456,
    "title": "無題 - メモ帳",
    "pid": 12345,
    "process_name": "notepad",
    "visible": true,
    "rect": { "x": 100, "y": 100, "width": 800, "height": 600 }
  }
]
```

---

## 現在フォアグラウンドのウィンドウを確認

AI が「今何が表示されているか」を把握するのに使う。

```bash
desktopwright foreground
desktopwright foreground --format json
```

---

## ウィンドウ状態の操作

```bash
# 最小化（最小化したウィンドウはキャプチャできない場合がある）
desktopwright window --target "メモ帳" --action minimize

# 最大化
desktopwright window --target "メモ帳" --action maximize

# リストア（通常サイズに戻す）
desktopwright window --target "メモ帳" --action restore
```

---

## Win32 固有の概念: HWND

Playwright の「ページ」に相当するのが HWND。

- HWND は OS が割り当てる整数値で、ウィンドウが閉じられるまで不変
- 同一プロセスが複数のウィンドウを持つことがある（例: ブラウザの複数タブ/ウィンドウ）
- 子ウィンドウ（コモンダイアログ等）は別の HWND を持つ
- `list` コマンドは最上位の可視ウィンドウを列挙する

### 同一プロセスの複数ウィンドウへの対処

```bash
# プロセス名だけでは複数一致する場合
desktopwright list --process chrome

# → HWND で特定のウィンドウを指定
desktopwright focus --hwnd <specific_hwnd>
```

---

## フォアグラウンドとフォーカスについて

- **フォアグラウンドウィンドウ**: 最前面にあり、キーボード入力を受け付けるウィンドウ
- `focus` コマンドはウィンドウをフォアグラウンドに移動する
- キーボード操作（`key`、`type`）の前に `focus` を実行すること
- クリック操作は `focus` なしでも動作するが、意図しないウィンドウに当たる可能性がある
