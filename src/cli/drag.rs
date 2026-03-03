use anyhow::{anyhow, Result};
use clap::Args;

use crate::cli::{parse_hwnd, ButtonArg, CoordModeArg};
use crate::core::platform::{InputController, UiAutomation};
use crate::core::types::TextMatchMode;

#[derive(Args, Debug)]
pub struct DragArgs {
    /// ドラッグ開始X座標
    #[arg(long)]
    pub from_x: Option<i32>,

    /// ドラッグ開始Y座標
    #[arg(long)]
    pub from_y: Option<i32>,

    /// ドラッグ終了X座標
    #[arg(long)]
    pub to_x: Option<i32>,

    /// ドラッグ終了Y座標
    #[arg(long)]
    pub to_y: Option<i32>,

    /// ドラッグ開始要素のテキスト（UIA で座標を自動取得）
    #[arg(long)]
    pub from_element: Option<String>,

    /// ドラッグ終了要素のテキスト（UIA で座標を自動取得）
    #[arg(long)]
    pub to_element: Option<String>,

    /// HWND直接指定
    #[arg(long)]
    pub hwnd: Option<String>,

    /// 座標系（window = ウィンドウ相対、screen = スクリーン絶対）
    #[arg(long, value_enum, default_value = "screen")]
    pub coord: CoordModeArg,

    /// マウスボタン
    #[arg(long, value_enum, default_value = "left")]
    pub button: ButtonArg,

    /// ドラッグ中の移動ステップ数（滑らかさ）
    #[arg(long, default_value_t = 10)]
    pub steps: u32,
}

#[derive(Args, Debug)]
pub struct MousedownArgs {
    /// X座標
    #[arg(long, short = 'x')]
    pub x: i32,
    /// Y座標
    #[arg(long, short = 'y')]
    pub y: i32,

    /// HWND直接指定（--coord window の場合に必要）
    #[arg(long)]
    pub hwnd: Option<String>,

    /// 座標系
    #[arg(long, value_enum, default_value = "screen")]
    pub coord: CoordModeArg,

    /// マウスボタン
    #[arg(long, value_enum, default_value = "left")]
    pub button: ButtonArg,
}

#[derive(Args, Debug)]
pub struct MouseupArgs {
    /// X座標
    #[arg(long, short = 'x')]
    pub x: i32,
    /// Y座標
    #[arg(long, short = 'y')]
    pub y: i32,

    /// HWND直接指定（--coord window の場合に必要）
    #[arg(long)]
    pub hwnd: Option<String>,

    /// 座標系
    #[arg(long, value_enum, default_value = "screen")]
    pub coord: CoordModeArg,

    /// マウスボタン
    #[arg(long, value_enum, default_value = "left")]
    pub button: ButtonArg,
}

pub fn run_drag(
    args: &DragArgs,
    automation: &dyn UiAutomation,
    input: &dyn InputController,
) -> Result<()> {
    let hwnd_val = args.hwnd.as_ref().map(|s| parse_hwnd(s)).transpose()?;

    let (from_x, from_y) = resolve_position(
        args.from_x, args.from_y, args.from_element.as_deref(), hwnd_val, automation,
    )?;
    let (to_x, to_y) = resolve_position(
        args.to_x, args.to_y, args.to_element.as_deref(), hwnd_val, automation,
    )?;

    input.mouse_button_down(from_x, from_y, args.button.into(), hwnd_val, args.coord.into())?;

    let steps = args.steps.max(1);
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let mx = from_x + ((to_x - from_x) as f64 * t) as i32;
        let my = from_y + ((to_y - from_y) as f64 * t) as i32;
        input.mouse_move(mx, my, hwnd_val, args.coord.into())?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    input.mouse_button_up(to_x, to_y, args.button.into(), hwnd_val, args.coord.into())?;

    eprintln!("ドラッグ: ({},{}) → ({},{})", from_x, from_y, to_x, to_y);
    Ok(())
}

pub fn run_mousedown(args: &MousedownArgs, input: &dyn InputController) -> Result<()> {
    let hwnd_val = args.hwnd.as_ref().map(|s| parse_hwnd(s)).transpose()?;
    input.mouse_button_down(args.x, args.y, args.button.into(), hwnd_val, args.coord.into())?;
    eprintln!("mousedown: ({},{})", args.x, args.y);
    Ok(())
}

pub fn run_mouseup(args: &MouseupArgs, input: &dyn InputController) -> Result<()> {
    let hwnd_val = args.hwnd.as_ref().map(|s| parse_hwnd(s)).transpose()?;
    input.mouse_button_up(args.x, args.y, args.button.into(), hwnd_val, args.coord.into())?;
    eprintln!("mouseup: ({},{})", args.x, args.y);
    Ok(())
}

/// --from-x/y と --from-element のどちらかから座標を解決する
fn resolve_position(
    x: Option<i32>,
    y: Option<i32>,
    element_text: Option<&str>,
    hwnd_val: Option<usize>,
    automation: &dyn UiAutomation,
) -> Result<(i32, i32)> {
    if let Some(text) = element_text {
        let hwnd = hwnd_val
            .ok_or_else(|| anyhow!("--from-element / --to-element には --hwnd が必要です"))?;
        let node = automation
            .find_element(hwnd, text, None, 0, TextMatchMode::Contains, 2000)?
            .ok_or_else(|| anyhow!("要素が見つかりません: {:?}", text))?;

        let rect = node.rect.ok_or_else(|| anyhow!("要素の矩形が取得できません"))?;
        // UIA 座標はスクリーン絶対座標
        return Ok((rect.x + rect.width / 2, rect.y + rect.height / 2));
    }

    match (x, y) {
        (Some(px), Some(py)) => Ok((px, py)),
        _ => Err(anyhow!("--from-x/--from-y または --from-element を指定してください")),
    }
}
