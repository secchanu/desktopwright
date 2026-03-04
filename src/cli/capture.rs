use anyhow::{Result, anyhow};
use clap::Args;
use image::DynamicImage;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::WindowTargetArgs;
use crate::core::platform::{ScreenCapture, WindowManager};
use crate::core::types::{CaptureOptions, ImageFormat, Rect};
use crate::output::print_json;

#[derive(Args, Debug)]
pub struct CaptureArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 出力ファイルパス（省略時はstdoutにバイナリ出力）
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// 出力フォーマット（png, jpeg, bmp）
    #[arg(long, default_value = "png")]
    pub format: ImageFormatArg,

    /// 部分領域: x座標（ウィンドウ相対）
    #[arg(long)]
    pub region_x: Option<i32>,

    /// 部分領域: y座標（ウィンドウ相対）
    #[arg(long)]
    pub region_y: Option<i32>,

    /// 部分領域: 幅
    #[arg(long)]
    pub region_width: Option<i32>,

    /// 部分領域: 高さ
    #[arg(long)]
    pub region_height: Option<i32>,

    /// 出力画像の最大幅（アスペクト比を維持してリサイズ）
    #[arg(long)]
    pub max_width: Option<u32>,

    /// 出力画像の最大高さ（アスペクト比を維持してリサイズ）
    #[arg(long)]
    pub max_height: Option<u32>,

    /// 差分検出モード: タイムアウトまで変化を待つ（ミリ秒）
    #[arg(long = "wait-for-diff")]
    pub wait_for_diff: Option<u64>,

    /// 差分検出の閾値（0.0〜1.0、デフォルト 0.05）
    #[arg(long, default_value = "0.05")]
    pub diff_threshold: f32,

    /// キャプチャ前の待機時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,

    /// カーソル位置を赤いクロスヘアでキャプチャ画像に描画する
    #[arg(long)]
    pub cursor: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum ImageFormatArg {
    Png,
    Jpeg,
    Bmp,
}

impl From<ImageFormatArg> for ImageFormat {
    fn from(f: ImageFormatArg) -> Self {
        match f {
            ImageFormatArg::Png => ImageFormat::Png,
            ImageFormatArg::Jpeg => ImageFormat::Jpeg,
            ImageFormatArg::Bmp => ImageFormat::Bmp,
        }
    }
}

/// --json 時の capture コマンドの出力
#[derive(Serialize)]
struct CaptureJsonResult {
    path: Option<String>,
    format: String,
    width: u32,
    height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    changed_region: Option<Rect>,
}

pub fn run_capture(
    args: &CaptureArgs,
    manager: &dyn WindowManager,
    capture: &dyn ScreenCapture,
    json: bool,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;

    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    let region = build_region(args);
    let options = CaptureOptions {
        region,
        max_width: args.max_width,
        max_height: args.max_height,
        format: args.format.into(),
        cursor: args.cursor,
    };

    let (img, changed_region) = if let Some(timeout_ms) = args.wait_for_diff {
        match capture.capture_diff(window.hwnd, timeout_ms, args.diff_threshold, &options)? {
            Some((img, diff)) => {
                if !json {
                    eprintln!(
                        "変化を検出: {} 領域, バウンディングボックス: {:?}",
                        diff.changed_regions.len(),
                        diff.bounding_box
                    );
                }
                (img, diff.bounding_box)
            }
            None => {
                eprintln!(
                    "タイムアウト: {}ms 以内に変化がありませんでした",
                    timeout_ms
                );
                return Ok(());
            }
        }
    } else {
        (capture.capture_window(window.hwnd, &options)?, None)
    };

    let width = img.width();
    let height = img.height();
    let path = args.output.as_ref().map(|p| p.display().to_string());
    let format_str = format!("{:?}", args.format).to_lowercase();

    save_or_output_image(img, args)?;

    if json {
        print_json(&CaptureJsonResult {
            path,
            format: format_str,
            width,
            height,
            changed_region,
        })?;
    }

    Ok(())
}

fn build_region(args: &CaptureArgs) -> Option<Rect> {
    match (
        args.region_x,
        args.region_y,
        args.region_width,
        args.region_height,
    ) {
        (Some(x), Some(y), Some(w), Some(h)) => Some(Rect {
            x,
            y,
            width: w,
            height: h,
        }),
        _ => None,
    }
}

fn save_or_output_image(img: DynamicImage, args: &CaptureArgs) -> Result<()> {
    if let Some(path) = &args.output {
        img.save(path).map_err(|e| anyhow!("画像保存失敗: {}", e))?;
        eprintln!("保存: {}", path.display());
    } else {
        let mut buf = Vec::new();
        let format = match args.format {
            ImageFormatArg::Png => image::ImageFormat::Png,
            ImageFormatArg::Jpeg => image::ImageFormat::Jpeg,
            ImageFormatArg::Bmp => image::ImageFormat::Bmp,
        };
        img.write_to(&mut std::io::Cursor::new(&mut buf), format)
            .map_err(|e| anyhow!("画像エンコード失敗: {}", e))?;
        std::io::stdout()
            .write_all(&buf)
            .map_err(|e| anyhow!("stdout出力失敗: {}", e))?;
    }
    Ok(())
}
