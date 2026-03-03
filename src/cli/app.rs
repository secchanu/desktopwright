use anyhow::{anyhow, Result};
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::core::platform::WindowManager;

#[derive(Args, Debug)]
pub struct LaunchArgs {
    /// 起動するアプリのパス
    pub path: String,

    /// アプリに渡す引数
    pub args: Vec<String>,

    /// 起動後にスリープする時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,
}

pub fn run_launch(args: &LaunchArgs) -> Result<()> {
    let child = std::process::Command::new(&args.path)
        .args(&args.args)
        .spawn()
        .map_err(|e| anyhow!("アプリの起動に失敗しました: {}: {}", args.path, e))?;

    eprintln!("起動しました (PID: {}): {}", child.id(), args.path);

    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    Ok(())
}

pub fn run_close(args: &CloseArgs, manager: &dyn WindowManager) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    manager.close_window(window.hwnd)?;
    eprintln!("ウィンドウを閉じました: \"{}\" (HWND: {})", window.title, window.hwnd);
    Ok(())
}
