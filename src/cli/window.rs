use anyhow::Result;
use clap::{Args, ValueEnum};

use crate::cli::WindowTargetArgs;
use crate::core::platform::WindowManager;
use crate::core::types::WindowState;

#[derive(Args, Debug)]
pub struct WindowArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 実行するアクション
    #[arg(long, short = 'a')]
    pub action: WindowAction,
}

#[derive(Args, Debug)]
pub struct ResizeArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// ウィンドウの幅（ピクセル）
    #[arg(long)]
    pub width: u32,

    /// ウィンドウの高さ（ピクセル）
    #[arg(long)]
    pub height: u32,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum WindowAction {
    /// 最小化
    Minimize,
    /// 最大化
    Maximize,
    /// リストア（通常サイズに戻す）
    Restore,
}

pub fn run_window(args: &WindowArgs, manager: &dyn WindowManager) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    let state = match args.action {
        WindowAction::Minimize => WindowState::Minimize,
        WindowAction::Maximize => WindowState::Maximize,
        WindowAction::Restore => WindowState::Restore,
    };
    manager.set_window_state(window.hwnd, state)?;
    eprintln!(
        "{:?}: \"{}\" (HWND: {})",
        args.action, window.title, window.hwnd
    );
    Ok(())
}

pub fn run_resize(args: &ResizeArgs, manager: &dyn WindowManager) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    manager.resize_window(window.hwnd, args.width, args.height)?;
    eprintln!(
        "リサイズ: \"{}\" (HWND: {}) → {}x{}",
        window.title, window.hwnd, args.width, args.height
    );
    Ok(())
}
