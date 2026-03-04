use anyhow::{Result, anyhow};
use clap::Args;

use crate::cli::{ButtonArg, CoordModeArg, parse_hwnd};
use crate::core::platform::InputController;

#[derive(Args, Debug)]
pub struct ClickArgs {
    /// X座標
    #[arg(long, short = 'x')]
    pub x: i32,

    /// Y座標
    #[arg(long, short = 'y')]
    pub y: i32,

    /// クリックボタン
    #[arg(long, short = 'b', default_value = "left")]
    pub button: ButtonArg,

    /// ダブルクリック
    #[arg(long)]
    pub double: bool,

    /// 座標系（screen: スクリーン絶対座標、window: ウィンドウ相対座標）
    #[arg(long, default_value = "screen")]
    pub coord: CoordModeArg,

    /// ウィンドウ相対座標を使う場合のHWND
    #[arg(long)]
    pub hwnd: Option<String>,

    /// クリック前の待機時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,

    /// PostMessage で WM_LBUTTONDOWN/UP を直接 HWND のメッセージキューに注入する。
    /// SendInput（デフォルト）ではフォアグラウンド状態に依存するが、このフラグを
    /// 使うとウィンドウのアクティブ状態に関係なくクリックを届けることができる。
    /// --hwnd が必須。座標は物理クライアント座標（--coord window）で指定する。
    #[arg(long)]
    pub direct: bool,
}

#[derive(Args, Debug)]
pub struct MoveArgs {
    /// X座標
    #[arg(long, short = 'x')]
    pub x: i32,

    /// Y座標
    #[arg(long, short = 'y')]
    pub y: i32,

    /// 座標系
    #[arg(long, default_value = "screen")]
    pub coord: CoordModeArg,

    /// ウィンドウ相対座標を使う場合のHWND
    #[arg(long)]
    pub hwnd: Option<String>,
}

pub fn run_click(args: &ClickArgs, input: &dyn InputController) -> Result<()> {
    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    let hwnd = args.hwnd.as_ref().map(|s| parse_hwnd(s)).transpose()?;

    if args.direct {
        let hwnd = hwnd.ok_or_else(|| anyhow!("--direct には --hwnd が必要です"))?;
        input.post_message_click(args.x, args.y, args.button.into(), hwnd, args.coord.into())?;
    } else {
        input.mouse_click(
            args.x,
            args.y,
            args.button.into(),
            args.double,
            hwnd,
            args.coord.into(),
        )?;
    }
    Ok(())
}

pub fn run_move(args: &MoveArgs, input: &dyn InputController) -> Result<()> {
    let hwnd = args.hwnd.as_ref().map(|s| parse_hwnd(s)).transpose()?;
    input.mouse_move(args.x, args.y, hwnd, args.coord.into())?;
    Ok(())
}
