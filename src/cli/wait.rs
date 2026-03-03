use anyhow::Result;
use clap::Args;
use std::time::{Duration, Instant};

use crate::cli::WindowTargetArgs;
use crate::core::platform::WindowManager;

#[derive(Args, Debug)]
pub struct WaitArgs {
    /// 待機時間（ミリ秒）
    pub ms: u64,
}

#[derive(Args, Debug)]
pub struct WaitForWindowArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// タイムアウト時間（ミリ秒、デフォルト 10000）
    #[arg(long, default_value = "10000")]
    pub timeout: u64,

    /// ポーリング間隔（ミリ秒、デフォルト 200）
    #[arg(long, default_value = "200")]
    pub interval_ms: u64,
}

pub fn run_wait(args: &WaitArgs) -> Result<()> {
    std::thread::sleep(Duration::from_millis(args.ms));
    Ok(())
}

pub fn run_wait_for_window(
    args: &WaitForWindowArgs,
    manager: &dyn WindowManager,
    json: bool,
) -> Result<()> {
    let target = args.window.resolve()?;
    let deadline = Instant::now() + Duration::from_millis(args.timeout);

    loop {
        if let Ok(mut windows) = manager.find_windows(&target) {
            if !windows.is_empty() {
                let w = windows.remove(0);
                if json {
                    println!("{}", serde_json::to_string_pretty(&w)?);
                } else {
                    println!(
                        "ウィンドウが見つかりました: \"{}\" (HWND: {})",
                        w.title, w.hwnd
                    );
                }
                return Ok(());
            }
        }

        if Instant::now() >= deadline {
            anyhow::bail!(
                "タイムアウト: {}ms 以内にウィンドウが見つかりませんでした",
                args.timeout
            );
        }

        std::thread::sleep(Duration::from_millis(args.interval_ms));
    }
}
