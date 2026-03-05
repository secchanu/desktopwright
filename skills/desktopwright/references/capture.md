# Capture

スクリーンキャプチャと差分検出の使い方を説明する。

---

## 基本キャプチャ

```bash
# ファイルに保存
desktopwright capture --target "メモ帳" --output screen.png

# stdout に出力（AI への直接渡し）
desktopwright capture --target "メモ帳" > screen.png

# HWND で指定（最も確実）
desktopwright capture --hwnd 132456 --output screen.png
```

### 出力フォーマット

```bash
# PNG（デフォルト）
desktopwright capture --target "メモ帳" --format png --output screen.png

# JPEG
desktopwright capture --target "メモ帳" --format jpeg --output screen.jpg

# BMP
desktopwright capture --target "メモ帳" --format bmp --output screen.bmp
```

---

## カーソル位置の可視化（--cursor）

`--cursor` フラグを付けると、現在のマウスカーソル位置に赤いオーバーレイを描画してキャプチャする。
`move` でカーソルを移動した直後にこのオプションで撮影することで、**座標が本当に目的の場所に届いているか**を視覚的に確認できる。

**オーバーレイの構成**:
- 中心に 1px の白マーカー（正確なカーソル位置）
- 中心 ±2px はクリアゾーン（描画なし）— 対象ピクセルを隠さないため
- 上下左右に画像端まで伸びるダッシュ腕（3px on / 2px off）でカーソルの行/列を示す
- 腕上にフィボナッチ数列の距離に垂直目盛りを配置（3, 5, 8, 13, 21, 34, 55, 89px）
  - 目盛りの半幅は距離に応じて拡大（2→5px）して遠近感を表現

クリアゾーンにより、カーソルが 1-2px ズレている場合でもターゲットピクセルが透けて見える。
フィボナッチ目盛りは「ターゲットが 34px 目盛りと 55px 目盛りの間にある」といった粗い位置合わせに使う。

> **精度について**: 必要な精度はアプリによって異なる（小さいボタンや 1px のターゲットは 1px 単位の精度が必要な場合がある）。
> クリック後の画面変化や座標報告で実際のズレを確認し、必要であれば再度 `move → capture --cursor` で追い込むこと。

```bash
# カーソルを移動してから、オーバーレイ付きでキャプチャして確認
desktopwright move --x 107 --y 789 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --cursor --output verify.png
# verify.png を確認し、赤い中心十字が目的の要素の上にあるか確認する
# ズレていれば目盛りで距離を読み取り、move の座標を調整して再試行する
```

---

## 部分領域キャプチャ

ウィンドウの特定領域のみをキャプチャする。座標は `capture` で取得した画像の座標系（xcap 画像座標系）と同じ。

```bash
desktopwright capture \
  --target "メモ帳" \
  --region-x 0 \
  --region-y 0 \
  --region-width 400 \
  --region-height 300 \
  --output top-left.png
```

---

## 画像サイズの制限

大きなウィンドウを AI に渡す際にサイズを縮小する。アスペクト比は維持される。

```bash
# 最大幅 800px に縮小
desktopwright capture --target "Chrome" --max-width 800 --output small.png

# 最大 1024x768 に縮小
desktopwright capture --target "Chrome" --max-width 1024 --max-height 768 --output small.png
```

> **警告**: `--max-width` / `--max-height` でリサイズした画像の座標は、そのまま `click --coord window` に使えない。
> 縮小後の画像上の座標 (sx, sy) を元のウィンドウ座標に戻すには `sx * (元幅 / 縮小幅)` のスケール逆変換が必要。
> **座標指定のために使う画像は `--max-width` / `--max-height` を付けずにキャプチャすること。**

---

## キャプチャ前の待機

操作直後のアニメーションや描画完了を待つ。

```bash
# 500ms 待ってからキャプチャ
desktopwright capture --target "メモ帳" --delay 500 --output after.png
```

---

## 差分検出モード

一定時間内に画面に変化があった場合のみ、変化した領域の画像を返す。

**使いどころ**:
- 操作（クリック、キー入力）後に「画面が更新されたか」を確認する
- ダイアログや通知の出現を待つ
- ページ遷移やロードの完了を検知する

```bash
# 3000ms 以内に変化があれば変化領域を返す
desktopwright capture \
  --target "Chrome" \
  --wait-for-diff 3000 \
  --output diff.png

# 変化がなかった場合: 何も出力されず、終了コード 0
```

### 差分閾値の調整

微細なノイズ（カーソル点滅、アンチエイリアス）を無視するための閾値。

```bash
# 閾値を上げてノイズを無視（デフォルト 0.05）
desktopwright capture \
  --target "Chrome" \
  --wait-for-diff 3000 \
  --diff-threshold 0.1 \
  --output diff.png
```

### 差分検出の stderr 出力

変化が検出された場合、変化情報が stderr に出力される:
```
変化を検出: 1 領域, バウンディングボックス: Some(Rect { x: 100, y: 200, width: 300, height: 50 })
```

変化がなかった場合:
```
タイムアウト: 3000ms 以内に変化がありませんでした
```

---

## 組み合わせパターン

### 操作後の変化を確認する

```bash
# クリック後に画面更新を待って確認
desktopwright click --x 400 --y 300 --coord window --hwnd 132456
desktopwright capture --hwnd 132456 --wait-for-diff 2000 --output result.png
```

### ホバー状態のキャプチャ

```bash
# マウスを移動してツールチップが出るのを待つ
desktopwright move --x 400 --y 300
desktopwright capture --target "メモ帳" --delay 1000 --output hover.png
```

---

## 注意事項

### キャプチャがブロックされる場合

DRM 保護や一部のセキュリティソフトが原因でキャプチャが黒画像になる場合がある。
エラーメッセージ: `キャプチャがブロックされています（DRMまたはセキュリティ保護）`

### 最小化されたウィンドウ

最小化されたウィンドウはキャプチャできない場合がある。
事前に `restore` または `focus` でウィンドウを表示状態にすること:
```bash
desktopwright window --target "メモ帳" --action restore
desktopwright capture --target "メモ帳" --output screen.png
```
