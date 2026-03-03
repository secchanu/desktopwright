# Troubleshooting

よくある失敗パターンと対処法を説明する。

---

## ウィンドウが見つからない

### エラー: `ウィンドウが見つかりません`

```
Error: ウィンドウが見つかりません: Title("メモ帳")
```

**原因と対処**:
1. タイトルの大文字・小文字、スペースを確認する（部分一致なので完全一致は不要）
2. `list` コマンドで実際のタイトルを確認する
3. ウィンドウが最小化されている場合は見つかることがあるが、タイトルが変わっていることも

```bash
# 実際のウィンドウ一覧を確認
desktopwright list
desktopwright list | grep -i notepad  # 部分一致で検索
```

### エラー: `複数のウィンドウが一致します`

```
Error: 3件一致しました。HWNDで直接指定してください:
"設定" (HWND: 123456)
"設定" (HWND: 234567)
"設定" (HWND: 345678)
```

**対処**: エラーメッセージのHWNDから目的のウィンドウを特定して直接指定する。

```bash
desktopwright focus --hwnd 123456
```

---

## キャプチャが黒画像になる

### エラー: `キャプチャがブロックされています`

DRM 保護のあるアプリ（映像配信サービス、一部のセキュリティソフト等）はキャプチャをブロックする。
回避手段はない。

**確認手順**:
- まず `capture` を実行してエラーメッセージを確認する
- エラーにならず黒画像が保存された場合も、当ツールでは検出できないケースがある

### 最小化されているウィンドウ

最小化状態ではキャプチャできない場合がある。

```bash
# 先にリストアしてからキャプチャ
desktopwright window --target "アプリ" --action restore
desktopwright capture --target "アプリ" --output screen.png
```

---

## 座標がズレる (DPIスケーリング)

### 症状

125%・150%などの高DPI設定環境で、`capture` 画像上の座標と `click --coord window` 座標がズレる。

**原因**: `capture` は物理ピクセル座標で画像を返すが、`--coord window` は Win32 ロジカル座標を
使用する。125% DPI では物理座標 ÷ 1.25 = ロジカル座標になる。

gpui（Zed フレームワーク）など Per-Monitor DPI-Aware なアプリでは特にこのズレが発生する。

**対処**: キャプチャ画像上の座標を DPI スケール係数で割って click 座標に使う。

```bash
# DPI 125% の場合: 画像上の (760, 508) → click 座標は (608, 406)
# 計算式: click_x = capture_x / 1.25, click_y = capture_y / 1.25

desktopwright click --x 608 --y 406 --coord window --hwnd <hwnd>
```

DPI スケール係数の確認方法：
- Windows 設定 → ディスプレイ → 拡大縮小とレイアウト → テキスト、アプリ、その他の項目のサイズを変更する
- 125% → 係数 1.25、150% → 係数 1.5、100% → 係数 1.0（ズレなし）

**代替手段**: UIA が使えるアプリなら `click-element --text "ボタン名"` で座標不要のクリックが可能。

---

## UAC 昇格プロセスに操作できない

### 症状

管理者として実行されているウィンドウに対して操作が効かない、または `focus` が失敗する。

**原因**: UAC の UIPI (User Interface Privilege Isolation) により、低権限プロセスから高権限プロセスの UI に直接操作を送ることができない。

**対処**: desktopwright 自体を管理者として実行する。
（「管理者として実行」で PowerShell / コマンドプロンプトを起動し、その中から実行する）

---

## キー入力が別のウィンドウに飛ぶ

### 症状

`key` や `type` で入力したテキストが、意図したウィンドウ以外に入力される。

**原因**:
- `focus` を実行してからキー入力するまでに別のウィンドウがフォーカスを奪った
- グローバルホットキーが横取りした

**対処**:
1. `focus` の直後に `key`/`type` を実行する（間に他のコマンドを挟まない）
2. 操作前に `foreground` で現在のフォアグラウンドウィンドウを確認する

```bash
# フォーカス直後に操作
desktopwright focus --target "メモ帳"
desktopwright type "テキスト"  # フォーカスと入力を連続して実行
```

---

## Electron アプリ（VSCode、Chrome 等）の UI ツリーが不完全

### 症状

`ui-tree` で取得したツリーが浅い、または要素が見つからない。

**原因**: Electron（Chromium ベース）アプリは通常の Win32 UI Automation が内部 Web ページ階層まで届かない場合がある。

**対処**:
- スクリーン座標ベースでキャプチャして目視確認する
- `capture` でスクリーンショットを撮って AI に解析させる

---

## gpui（Zed フレームワーク）アプリが `list` に表示されない

### 症状

`list` コマンドでウィンドウが表示されない、または `ウィンドウが見つかりません` エラーが出る。

**原因**: gpui アプリ（Zed、sashiki 等）はウィンドウタイトルを空文字列にする。`list` コマンドはデフォルトで空タイトルのウィンドウを非表示にするため、何も表示されない。

**HWND の探し方**:

```bash
# プロセス名で絞り込む（空タイトルのウィンドウも表示される）
desktopwright list --process sashiki

# すべてのウィンドウを表示（タイトルなしも含む）
desktopwright list --all
```

出力例（Class 列が `Zed::Window` になる）:

```
HWND         PID      Process              Class                     Title
----------------------------------------------------------------------------------------------------
2690012      80292    sashiki              Zed::Window
3014716      80292    sashiki              Zed::Window
```

HWND を特定したら `--hwnd` で直接指定して操作する:

```bash
desktopwright capture --hwnd 2690012 --output screen.png
desktopwright focus --hwnd 2690012
```

### UI Automation が使えない

gpui は Win32 UI Automation に対応していないため、`ui-tree`/`snapshot`/`click-element` は最小限の情報しか返さない。

**操作方法**: `capture` でスクリーンショットを撮り、AI に座標を特定させてから `click --coord window` で操作する。DPI スケーリングに注意（上記「座標がズレる」参照）。

---

## テキスト入力後にクリップボードが空になる

### 仕様

`type` コマンドはクリップボード貼り付け方式でテキストを入力するため、実行後にクリップボードの内容が変更される。

**対処**: 重要なクリップボード内容は `type` コマンドの前に退避する。

---

## アニメーション中のキャプチャ

### 症状

メニューを開いた直後やウィンドウ遷移中にキャプチャすると、中間状態が写る。

**対処**:
- `--delay` で適切な待機時間を設定する（200〜500ms が目安）
- または `--wait-for-diff` で安定状態を検知する（変化が収まるまで待てないため不適な場合もある）

```bash
# メニューを開いてアニメーション完了後にキャプチャ
desktopwright key alt+f
desktopwright capture --target "アプリ" --delay 300 --output menu.png
```

---

## リモートデスクトップ / 仮想マシン上での実行

一部の API が RDP セッション内で期待通りに動かない場合がある。
特に WGC (Windows Graphics Capture) を使うキャプチャが問題になることがある。
PrintWindow ベースのフォールバックが有効な場合は自動的に使われる。
