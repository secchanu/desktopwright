use anyhow::{Result, anyhow};
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::cli::ui_tree::{find_node_by_ref, parse_ref_str};
use crate::core::platform::{InputController, UiAutomation, WindowManager};
use crate::core::types::{ClickElementResult, CoordMode, MouseButton, TextMatchMode};
use crate::output::print_json;

#[derive(Args, Debug)]
pub struct ClickElementArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// クリックする要素のテキスト（アクセシブル名）。--ref と排他
    #[arg(long, required_unless_present = "ref_id")]
    pub text: Option<String>,

    /// snapshot コマンドで表示された ref 番号でクリック（例: e5）。--text と排他
    #[arg(long = "ref")]
    pub ref_id: Option<String>,

    /// コントロールタイプでフィルタ（button, edit, checkbox, link, listitem, menuitem 等）
    #[arg(long)]
    pub role: Option<String>,

    /// テキストマッチモード（contains または exact）
    #[arg(long, value_enum, default_value = "contains")]
    pub r#match: MatchModeArg,

    /// 複数マッチした場合のインデックス（0始まり）
    #[arg(long, default_value_t = 0)]
    pub index: usize,

    /// 要素検索のタイムアウト（ミリ秒）
    #[arg(long, default_value_t = 2000)]
    pub timeout: u64,

    /// クリック前の待機時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,

    /// ダブルクリック
    #[arg(long)]
    pub double: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchModeArg {
    Contains,
    Exact,
}

pub fn run_click_element(
    args: &ClickElementArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
    input: &dyn InputController,
    json: bool,
) -> Result<()> {
    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    let window = manager.find_window(&args.window.resolve()?)?;

    let node = if let Some(ref_str) = &args.ref_id {
        let ref_num = parse_ref_str(ref_str)
            .ok_or_else(|| anyhow!("--ref の形式が不正です（例: e5）: {:?}", ref_str))?;
        let tree = automation.get_ui_tree(window.hwnd, Some(20))?;
        find_node_by_ref(&tree, ref_num)
            .ok_or_else(|| anyhow!("ref {} に対応する要素が見つかりません", ref_str))?
    } else {
        let text = args
            .text
            .as_deref()
            .expect("required_unless_present=ref_id がテキストを保証する");
        let match_mode = match args.r#match {
            MatchModeArg::Contains => TextMatchMode::Contains,
            MatchModeArg::Exact => TextMatchMode::Exact,
        };
        automation
            .find_element(
                window.hwnd,
                text,
                args.role.as_deref(),
                args.index,
                match_mode,
                args.timeout,
            )?
            .ok_or_else(|| {
                anyhow!(
                    "要素が見つかりません: text={:?}, role={:?}, index={}",
                    text,
                    args.role,
                    args.index
                )
            })?
    };

    let rect = node
        .rect
        .ok_or_else(|| anyhow!("要素の矩形が取得できません: {:?}", node.name))?;

    // UI Automationの座標はスクリーン絶対座標
    let click_x = rect.x + rect.width / 2;
    let click_y = rect.y + rect.height / 2;

    input.mouse_click(
        click_x,
        click_y,
        MouseButton::Left,
        args.double,
        None,
        CoordMode::Screen,
    )?;

    let result = ClickElementResult {
        name: node.name,
        role: node.control_type,
        class_name: node.class_name,
        rect,
        click_x,
        click_y,
    };

    if json {
        print_json(&result)?;
    } else {
        eprintln!(
            "クリック: {:?} [{}] ({}, {})",
            result.name, result.role, result.click_x, result.click_y
        );
    }

    Ok(())
}
