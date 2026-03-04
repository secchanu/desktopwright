mod cli;
mod core;
mod output;
mod platform;
mod version;

use anyhow::Result;
use clap::Parser;

use cli::{
    Cli, Commands,
    app::{run_close, run_launch},
    capture::run_capture,
    click_element::run_click_element,
    drag::{run_drag, run_mousedown, run_mouseup},
    element_action::{run_check, run_dialog_accept, run_dialog_dismiss, run_select, run_uncheck},
    focus::run_focus,
    get_text::run_get_text,
    install::run_install,
    key::{run_key, run_keydown, run_keyup},
    list::{run_foreground, run_list},
    mouse::{run_click, run_move},
    scroll::run_scroll,
    type_text::run_type,
    ui_tree::{run_snapshot, run_ui_tree},
    wait::{run_wait, run_wait_for_window},
    window::{run_resize, run_window},
};

fn main() -> Result<()> {
    // xcap（物理ピクセル）と Win32 座標系（ClientToScreen + SendInput）を一致させるため
    // プロセス全体を System DPI-Aware に設定する。
    // これにより、キャプチャ画像上の座標をそのままクリック座標として使用できる。
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;
        let _ = SetProcessDPIAware();
    }

    let cli = Cli::parse();
    let json = cli.json;
    let platform = platform::create_platform();

    match &cli.command {
        Commands::List(args) => {
            run_list(args, platform.window_manager.as_ref(), json)?;
        }
        Commands::Capture(args) => {
            run_capture(
                args,
                platform.window_manager.as_ref(),
                platform.screen_capture.as_ref(),
                json,
            )?;
        }
        Commands::Focus(args) => {
            run_focus(args, platform.window_manager.as_ref(), json)?;
        }
        Commands::Window(args) => {
            run_window(args, platform.window_manager.as_ref())?;
        }
        Commands::Resize(args) => {
            run_resize(args, platform.window_manager.as_ref())?;
        }
        Commands::Click(args) => {
            run_click(args, platform.input_controller.as_ref())?;
        }
        Commands::Move(args) => {
            run_move(args, platform.input_controller.as_ref())?;
        }
        Commands::Drag(args) => {
            run_drag(
                args,
                platform.ui_automation.as_ref(),
                platform.input_controller.as_ref(),
            )?;
        }
        Commands::Mousedown(args) => {
            run_mousedown(args, platform.input_controller.as_ref())?;
        }
        Commands::Mouseup(args) => {
            run_mouseup(args, platform.input_controller.as_ref())?;
        }
        Commands::Scroll(args) => {
            run_scroll(args, platform.input_controller.as_ref())?;
        }
        Commands::Key(args) => {
            run_key(args, platform.input_controller.as_ref())?;
        }
        Commands::Keydown(args) => {
            run_keydown(args, platform.input_controller.as_ref())?;
        }
        Commands::Keyup(args) => {
            run_keyup(args, platform.input_controller.as_ref())?;
        }
        Commands::Type(args) => {
            run_type(args, platform.input_controller.as_ref())?;
        }
        Commands::UiTree(args) => {
            run_ui_tree(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::Snapshot(args) => {
            run_snapshot(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::Foreground(args) => {
            run_foreground(args, platform.window_manager.as_ref(), json)?;
        }
        Commands::ClickElement(args) => {
            run_click_element(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
                platform.input_controller.as_ref(),
                json,
            )?;
        }
        Commands::Check(args) => {
            run_check(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::Uncheck(args) => {
            run_uncheck(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::Select(args) => {
            run_select(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::DialogAccept(args) => {
            run_dialog_accept(
                args,
                platform.window_manager.as_ref(),
                platform.input_controller.as_ref(),
            )?;
        }
        Commands::DialogDismiss(args) => {
            run_dialog_dismiss(
                args,
                platform.window_manager.as_ref(),
                platform.input_controller.as_ref(),
            )?;
        }
        Commands::Wait(args) => {
            run_wait(args)?;
        }
        Commands::WaitForWindow(args) => {
            run_wait_for_window(args, platform.window_manager.as_ref(), json)?;
        }
        Commands::Launch(args) => {
            run_launch(args)?;
        }
        Commands::Close(args) => {
            run_close(args, platform.window_manager.as_ref())?;
        }
        Commands::GetText(args) => {
            run_get_text(
                args,
                platform.window_manager.as_ref(),
                platform.ui_automation.as_ref(),
            )?;
        }
        Commands::Install(args) => {
            run_install(args)?;
        }
    }

    Ok(())
}
