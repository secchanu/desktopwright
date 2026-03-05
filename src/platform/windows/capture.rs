use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::UI::WindowsAndMessaging::GetPhysicalCursorPos;
use xcap::Window;

use crate::core::error::DesktopError;
use crate::core::platform::ScreenCapture;
use crate::core::types::{CaptureOptions, DiffResult, Rect};

use super::to_hwnd;

pub struct WindowsScreenCapture;

impl WindowsScreenCapture {
    pub fn new() -> Self {
        WindowsScreenCapture
    }

    /// xcapのWindowリストからHWNDに一致するウィンドウを探す
    fn find_xcap_window(hwnd: usize) -> Result<Window> {
        let windows = Window::all().map_err(|e| anyhow!("ウィンドウ一覧取得失敗: {}", e))?;
        windows
            .into_iter()
            .find(|w| w.id().ok().map_or(false, |id| id as usize == hwnd))
            .ok_or_else(|| {
                DesktopError::WindowNotFound(format!(
                    "HWND {} のxcapウィンドウが見つかりません",
                    hwnd
                ))
                .into()
            })
    }

    /// xcapでキャプチャした画像をimage::DynamicImageに変換する
    fn xcap_image_to_dynamic(img: xcap::image::RgbaImage) -> DynamicImage {
        let (width, height) = img.dimensions();
        let raw = img.into_raw();
        let buf: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, raw).expect("画像バッファ変換失敗");
        DynamicImage::ImageRgba8(buf)
    }

    /// 部分領域クロップを適用する
    fn apply_region(img: DynamicImage, region: &Rect) -> DynamicImage {
        img.crop_imm(
            region.x.max(0) as u32,
            region.y.max(0) as u32,
            region.width as u32,
            region.height as u32,
        )
    }

    /// 最大サイズにリサイズする（アスペクト比を維持）
    fn apply_max_size(
        img: DynamicImage,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> DynamicImage {
        let (w, h) = (img.width(), img.height());
        let scale_w = max_width.map(|mw| mw as f32 / w as f32).unwrap_or(1.0);
        let scale_h = max_height.map(|mh| mh as f32 / h as f32).unwrap_or(1.0);
        let scale = scale_w.min(scale_h);

        if scale < 1.0 {
            let new_w = (w as f32 * scale) as u32;
            let new_h = (h as f32 * scale) as u32;
            img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    }

    /// カーソル位置を取得してキャプチャ画像に赤いクロスヘアを描画する
    ///
    /// xcap はクライアント領域を物理ピクセル座標でキャプチャする。
    /// GetPhysicalCursorPos で DPI 非依存の物理カーソル座標を取得し、
    /// DWM フレームと xcap 画像サイズの差分からクライアント先頭オフセットを算出して
    /// xcap 画像内の正確な座標を求める。
    fn draw_cursor_overlay(img: &mut DynamicImage, hwnd: usize) {
        // GetPhysicalCursorPos はプロセスの DPI 設定に依らず常に物理座標を返す
        let mut cursor = POINT::default();
        if unsafe { GetPhysicalCursorPos(&mut cursor) }.is_err() {
            return;
        }

        let hwnd_win32 = to_hwnd(hwnd);
        let mut frame = RECT::default();
        if unsafe {
            DwmGetWindowAttribute(
                hwnd_win32,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut frame as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            )
        }
        .is_err()
        {
            return;
        }

        // xcap はクライアント領域にクロップするため、画像の原点はクライアント領域の
        // 左上（物理）になる。DWM フレームとクライアントの差分（上辺の非クライアント領域）を
        // 画像サイズの差分から求めてオフセットとして引く。
        // x 方向: DWM フレーム左 = クライアント左（影とフレーム厚が相殺）なので通常 0
        // y 方向: Win11 では上部に 1px 程度の非クライアント領域が存在する
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;
        let frame_w = frame.right - frame.left;
        let frame_h = frame.bottom - frame.top;
        let left_inset = frame_w - img_w; // 通常 0
        let top_inset = frame_h - img_h; // 通常 1（Win11 非最大化時）

        let cx = cursor.x - frame.left - left_inset;
        let cy = cursor.y - frame.top - top_inset;

        // カーソルがウィンドウ外ならスキップ
        if cx < 0 || cy < 0 || cx >= img_w || cy >= img_h {
            return;
        }

        let Some(rgba_img) = img.as_mut_rgba8() else {
            return;
        };

        let red = Rgba([255u8, 0, 0, 255]);

        // フィボナッチ目盛り: (中心からの距離px, 目盛りの半幅px)
        // 近いほど密・小さく、遠いほど疎・大きくして読みやすくする
        let ticks: &[(i32, i32)] = &[
            (3, 2),
            (5, 2),
            (8, 3),
            (13, 3),
            (21, 4),
            (34, 4),
            (55, 5),
            (89, 5),
        ];

        // 画像端まで伸びるダッシュ腕（3px on / 2px off）
        // d=1,2 はスキップして中心周辺に 2px のクリアゾーンを設ける。
        // これにより ±2px 以内の背景ピクセル（クリック対象など）が透けて見え、
        // 1px 単位のズレを目視で確認できる。
        for d in 3..img_w.max(img_h) {
            if (d % 5) < 3 {
                let pairs: [(i32, i32); 4] =
                    [(cx + d, cy), (cx - d, cy), (cx, cy + d), (cx, cy - d)];
                for (px, py) in pairs {
                    if px >= 0 && px < img_w && py >= 0 && py < img_h {
                        rgba_img.put_pixel(px as u32, py as u32, red);
                    }
                }
            }
        }

        // 中心の 1px 白マーカー（正確なカーソル位置を示す）
        // 白にすることで背景色に依らず視認できる。
        // ±2px のクリアゾーンと組み合わせて、中心付近のターゲットピクセルを隠さない。
        let white = Rgba([255u8, 255u8, 255u8, 255]);
        if cx >= 0 && cx < img_w && cy >= 0 && cy < img_h {
            rgba_img.put_pixel(cx as u32, cy as u32, white);
        }

        // フィボナッチ目盛りを腕に垂直に描く
        for &(dist, half) in ticks {
            for sign in [-1i32, 1] {
                // 水平腕の目盛り（垂直方向の線）
                let px = cx + sign * dist;
                if px >= 0 && px < img_w {
                    for dy in -half..=half {
                        let py = cy + dy;
                        if py >= 0 && py < img_h {
                            rgba_img.put_pixel(px as u32, py as u32, red);
                        }
                    }
                }
                // 垂直腕の目盛り（水平方向の線）
                let py = cy + sign * dist;
                if py >= 0 && py < img_h {
                    for dx in -half..=half {
                        let px = cx + dx;
                        if px >= 0 && px < img_w {
                            rgba_img.put_pixel(px as u32, py as u32, red);
                        }
                    }
                }
            }
        }
    }

    /// 2枚の画像を比較して変化領域を検出する
    ///
    /// threshold: 0.0〜1.0。ピクセルの差分がこの値を超えたら変化とみなす。
    /// 微細なアンチエイリアスや点滅ノイズを除去するため閾値を使う。
    fn detect_diff(img1: &DynamicImage, img2: &DynamicImage, threshold: f32) -> DiffResult {
        let img1 = img1.to_rgba8();
        let img2 = img2.to_rgba8();

        let (w, h) = img1.dimensions();
        let (w2, h2) = img2.dimensions();

        // サイズ違いの場合は全域を変化とみなす
        if w != w2 || h != h2 {
            return DiffResult {
                changed: true,
                changed_regions: vec![Rect {
                    x: 0,
                    y: 0,
                    width: w as i32,
                    height: h as i32,
                }],
                bounding_box: Some(Rect {
                    x: 0,
                    y: 0,
                    width: w as i32,
                    height: h as i32,
                }),
            };
        }

        let threshold_u8 = (threshold * 255.0) as u8;
        let mut min_x = w as i32;
        let mut min_y = h as i32;
        let mut max_x: i32 = -1;
        let mut max_y: i32 = -1;

        for y in 0..h {
            for x in 0..w {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);
                // RGBチャンネルの最大差分で判定（アルファは無視）
                let diff = p1.0[..3]
                    .iter()
                    .zip(p2.0[..3].iter())
                    .map(|(&a, &b)| (a as i16 - b as i16).unsigned_abs() as u8)
                    .max()
                    .unwrap_or(0);

                if diff > threshold_u8 {
                    min_x = min_x.min(x as i32);
                    min_y = min_y.min(y as i32);
                    max_x = max_x.max(x as i32);
                    max_y = max_y.max(y as i32);
                }
            }
        }

        if max_x < 0 {
            return DiffResult {
                changed: false,
                changed_regions: vec![],
                bounding_box: None,
            };
        }

        let bbox = Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        };

        DiffResult {
            changed: true,
            changed_regions: vec![bbox],
            bounding_box: Some(bbox),
        }
    }
}

impl ScreenCapture for WindowsScreenCapture {
    fn capture_window(&self, hwnd: usize, options: &CaptureOptions) -> Result<DynamicImage> {
        let window = Self::find_xcap_window(hwnd)?;

        let xcap_img = window
            .capture_image()
            .map_err(|e| anyhow!("キャプチャ失敗: {}", e))?;

        // 黒画像チェック（DRMブロック検出）
        // 画像サイズが十分にある場合のみサンプリング
        let (iw, ih) = xcap_img.dimensions();
        if iw > 4 && ih > 4 {
            let sample_pixel = xcap_img.get_pixel(0, 0);
            if sample_pixel.0 == [0, 0, 0, 255] {
                let is_all_black = [(iw / 4, ih / 4), (iw / 2, ih / 2), (3 * iw / 4, 3 * ih / 4)]
                    .iter()
                    .all(|(x, y)| xcap_img.get_pixel(*x, *y).0 == [0, 0, 0, 255]);
                if is_all_black {
                    return Err(DesktopError::CaptureBlocked.into());
                }
            }
        }

        let mut img = Self::xcap_image_to_dynamic(xcap_img);

        // カーソルオーバーレイ（クロップ・リサイズ前に物理ピクセル座標で描画する）
        if options.cursor {
            Self::draw_cursor_overlay(&mut img, hwnd);
        }

        // 部分領域クロップ
        if let Some(region) = &options.region {
            img = Self::apply_region(img, region);
        }

        // 最大サイズリサイズ
        img = Self::apply_max_size(img, options.max_width, options.max_height);

        Ok(img)
    }

    fn capture_diff(
        &self,
        hwnd: usize,
        timeout_ms: u64,
        threshold: f32,
        options: &CaptureOptions,
    ) -> Result<Option<(DynamicImage, DiffResult)>> {
        // 初期フレームをキャプチャ（差分検出はフルサイズで比較する）
        let baseline_opts = CaptureOptions {
            region: options.region.clone(),
            max_width: None,
            max_height: None,
            format: options.format,
            cursor: false,
        };
        let baseline = self.capture_window(hwnd, &baseline_opts)?;

        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let poll_interval = Duration::from_millis(100);

        loop {
            std::thread::sleep(poll_interval);

            if start.elapsed() >= timeout {
                return Ok(None);
            }

            let current = self.capture_window(hwnd, &baseline_opts)?;
            let diff = Self::detect_diff(&baseline, &current, threshold);

            if diff.changed {
                // 変化領域のみをクロップして返す
                let result_img = if let Some(bbox) = &diff.bounding_box {
                    let cropped = current.crop_imm(
                        bbox.x as u32,
                        bbox.y as u32,
                        bbox.width as u32,
                        bbox.height as u32,
                    );
                    Self::apply_max_size(cropped, options.max_width, options.max_height)
                } else {
                    Self::apply_max_size(current, options.max_width, options.max_height)
                };

                return Ok(Some((result_img, diff)));
            }
        }
    }
}
