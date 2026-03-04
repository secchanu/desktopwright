use anyhow::Result;
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::core::platform::{InputController, UiAutomation, WindowManager};

// ────────────────────────────────────────────────────────────────
// check / uncheck
// ────────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// チェックする要素のテキスト（アクセシブル名）
    #[arg(long)]
    pub text: String,

    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 要素検索のタイムアウト（ミリ秒）
    #[arg(long, default_value_t = 2000)]
    pub timeout: u64,
}

#[derive(Args, Debug)]
pub struct UncheckArgs {
    /// チェック解除する要素のテキスト（アクセシブル名）
    #[arg(long)]
    pub text: String,

    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 要素検索のタイムアウト（ミリ秒）
    #[arg(long, default_value_t = 2000)]
    pub timeout: u64,
}

pub fn run_check(
    args: &CheckArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    automation.toggle_element(
        window.hwnd,
        &args.text,
        Some("checkbox"),
        true,
        args.timeout,
    )?;
    eprintln!("チェック: {:?} (HWND: {})", args.text, window.hwnd);
    Ok(())
}

pub fn run_uncheck(
    args: &UncheckArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    automation.toggle_element(
        window.hwnd,
        &args.text,
        Some("checkbox"),
        false,
        args.timeout,
    )?;
    eprintln!("チェック解除: {:?} (HWND: {})", args.text, window.hwnd);
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// select
// ────────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct SelectArgs {
    /// 選択するオプションのテキスト
    #[arg(long)]
    pub value: String,

    /// コンボボックス・リストボックスの名前（省略時は value で直接検索）
    #[arg(long)]
    pub element: Option<String>,

    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 要素検索のタイムアウト（ミリ秒）
    #[arg(long, default_value_t = 3000)]
    pub timeout: u64,
}

pub fn run_select(
    args: &SelectArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    automation.select_option(
        window.hwnd,
        args.element.as_deref(),
        &args.value,
        args.timeout,
    )?;
    eprintln!("選択: {:?} (HWND: {})", args.value, window.hwnd);
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// dialog-accept / dialog-dismiss
// ────────────────────────────────────────────────────────────────

#[derive(Args, Debug)]
pub struct DialogAcceptArgs {
    /// 対象ウィンドウ（省略時は現在フォアグラウンドのウィンドウ）
    #[command(flatten)]
    pub window: WindowTargetArgs,
}

#[derive(Args, Debug)]
pub struct DialogDismissArgs {
    /// 対象ウィンドウ（省略時は現在フォアグラウンドのウィンドウ）
    #[command(flatten)]
    pub window: WindowTargetArgs,
}

pub fn run_dialog_accept(
    args: &DialogAcceptArgs,
    manager: &dyn WindowManager,
    input: &dyn InputController,
) -> Result<()> {
    focus_if_any(&args.window, manager)?;
    input.send_key("enter")?;
    eprintln!("dialog-accept: Enter を送信しました");
    Ok(())
}

pub fn run_dialog_dismiss(
    args: &DialogDismissArgs,
    manager: &dyn WindowManager,
    input: &dyn InputController,
) -> Result<()> {
    focus_if_any(&args.window, manager)?;
    input.send_key("escape")?;
    eprintln!("dialog-dismiss: Escape を送信しました");
    Ok(())
}

/// ウィンドウが指定されていればフォーカスする（省略時はスキップ）
fn focus_if_any(window_args: &WindowTargetArgs, manager: &dyn WindowManager) -> Result<()> {
    if window_args.hwnd.is_some() || window_args.target.is_some() || window_args.process.is_some() {
        let window = manager.find_window(&window_args.resolve()?)?;
        manager.focus_window(window.hwnd)?;
    }
    Ok(())
}
