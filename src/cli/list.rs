use anyhow::Result;
use clap::Args;

use crate::core::platform::WindowManager;
use crate::core::types::WindowInfo;
use crate::output::{OutputFormat, print_json};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// 出力フォーマット（table または json）。グローバル --json フラグでも指定可能
    #[arg(long, default_value = "table")]
    pub format: OutputFormat,

    /// プロセス名でフィルタ（部分一致）
    #[arg(long)]
    pub process: Option<String>,

    /// タイトルでフィルタ（部分一致）
    #[arg(long)]
    pub title: Option<String>,

    /// タイトルが空のウィンドウも含めて表示する（gpui など UIA 非対応アプリの HWND 探索に使用）
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct ForegroundArgs {
    /// 出力フォーマット（table または json）。グローバル --json フラグでも指定可能
    #[arg(long, default_value = "json")]
    pub format: OutputFormat,
}

pub fn run_list(args: &ListArgs, manager: &dyn WindowManager, json: bool) -> Result<()> {
    let mut windows = manager.list_windows()?;

    // フィルタ適用
    if let Some(process) = &args.process {
        let p = process.to_lowercase();
        windows.retain(|w| w.process_name.to_lowercase().contains(&p));
    }
    if let Some(title) = &args.title {
        let t = title.to_lowercase();
        windows.retain(|w| w.title.to_lowercase().contains(&t));
    }
    // --all 未指定時はタイトルが空のウィンドウを除外する（ゴーストウィンドウを非表示にする）
    if !args.all && args.process.is_none() && args.title.is_none() {
        windows.retain(|w| !w.title.is_empty());
    }

    // グローバル --json または --format json のどちらでもJSON出力
    if json || matches!(args.format, OutputFormat::Json) {
        print_json(&windows)?;
    } else {
        print_window_table(&windows);
    }

    Ok(())
}

pub fn run_foreground(
    args: &ForegroundArgs,
    manager: &dyn WindowManager,
    json: bool,
) -> Result<()> {
    let window = manager.get_foreground_window()?;
    if json || matches!(args.format, OutputFormat::Json) {
        print_json(&window)?;
    } else {
        match &window {
            Some(w) => print_window_table(&[w.clone()]),
            None => println!("フォアグラウンドウィンドウなし"),
        }
    }
    Ok(())
}

fn print_window_table(windows: &[WindowInfo]) {
    if windows.is_empty() {
        println!("ウィンドウが見つかりません");
        return;
    }
    println!(
        "{:<12} {:<8} {:<20} {:<25} {}",
        "HWND", "PID", "Process", "Class", "Title"
    );
    println!("{}", "-".repeat(100));
    for w in windows {
        println!(
            "{:<12} {:<8} {:<20} {:<25} {}{}",
            w.hwnd,
            w.pid,
            truncate(&w.process_name, 20),
            truncate(&w.class_name, 25),
            truncate(&w.title, 50),
            if w.minimized { " [最小化]" } else { "" },
        );
    }
}

fn truncate(s: &str, max: usize) -> &str {
    // マルチバイト文字を考慮してバイト境界で切り詰める
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}
