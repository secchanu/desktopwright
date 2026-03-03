use anyhow::Result;
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::core::platform::WindowManager;
use crate::output::print_json;

#[derive(Args, Debug)]
pub struct FocusArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,
}

pub fn run_focus(args: &FocusArgs, manager: &dyn WindowManager, json: bool) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    manager.focus_window(window.hwnd)?;
    if json {
        print_json(&window)?;
    } else {
        eprintln!("フォーカス: \"{}\" (HWND: {})", window.title, window.hwnd);
    }
    Ok(())
}
