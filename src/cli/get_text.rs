use anyhow::Result;
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::core::platform::{UiAutomation, WindowManager};
use crate::core::types::TextMatchMode;

#[derive(Args, Debug)]
pub struct GetTextArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// 対象要素のテキスト（アクセシブル名）
    #[arg(long)]
    pub text: Option<String>,

    /// 対象要素のロール（button, edit, text 等）
    #[arg(long)]
    pub role: Option<String>,

    /// 完全一致モード（デフォルト: 部分一致）
    #[arg(long)]
    pub exact: bool,

    /// タイムアウト（ミリ秒、デフォルト 5000）
    #[arg(long, default_value = "5000")]
    pub timeout: u64,
}

pub fn run_get_text(
    args: &GetTextArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;

    let match_mode = if args.exact {
        TextMatchMode::Exact
    } else {
        TextMatchMode::Contains
    };
    let search_text = args.text.as_deref().unwrap_or("");

    let node = automation
        .find_element(
            window.hwnd,
            search_text,
            args.role.as_deref(),
            0,
            match_mode,
            args.timeout,
        )?
        .ok_or_else(|| anyhow::anyhow!("要素が見つかりませんでした: {:?}", search_text))?;

    // value（入力フィールドの現在値）があればそれを、なければアクセシブル名を出力する
    let output = node.value.as_deref().unwrap_or(&node.name);
    println!("{}", output);

    Ok(())
}
