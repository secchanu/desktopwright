use anyhow::Result;
use clap::Args;

use crate::cli::WindowTargetArgs;
use crate::core::platform::{UiAutomation, WindowManager};
use crate::core::types::UiNode;

#[derive(Args, Debug)]
pub struct UiTreeArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// ツリーの最大深さ（デフォルト 5）
    #[arg(long, default_value = "5")]
    pub max_depth: u32,

    /// 出力フォーマット（tree または json）
    #[arg(long, default_value = "tree")]
    pub format: UiTreeFormat,

    /// フォーカス中の要素のみ表示
    #[arg(long)]
    pub focused: bool,
}

#[derive(Args, Debug)]
pub struct SnapshotArgs {
    #[command(flatten)]
    pub window: WindowTargetArgs,

    /// ツリーの最大深さ（デフォルト 20）
    #[arg(long, default_value = "20")]
    pub max_depth: u32,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum UiTreeFormat {
    Tree,
    Json,
}

pub fn run_ui_tree(
    args: &UiTreeArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    if args.focused {
        match automation.get_focused_element()? {
            Some(node) => print_node_output(&node, &args.format),
            None => eprintln!("フォーカス中の要素がありません"),
        }
        return Ok(());
    }

    let window = manager.find_window(&args.window.resolve()?)?;
    let tree = automation.get_ui_tree(window.hwnd, Some(args.max_depth))?;
    print_node_output(&tree, &args.format);
    Ok(())
}

fn print_node_output(node: &UiNode, format: &UiTreeFormat) {
    match format {
        UiTreeFormat::Json => {
            println!("{}", serde_json::to_string_pretty(node).unwrap_or_default());
        }
        UiTreeFormat::Tree => {
            print_tree(node, "", true);
        }
    }
}

fn print_tree(node: &UiNode, prefix: &str, last: bool) {
    let connector = if last { "└── " } else { "├── " };
    let child_prefix = if last { "    " } else { "│   " };

    let label = format_node_label(node);
    println!("{}{}{}", prefix, connector, label);

    let full_prefix = format!("{}{}", prefix, child_prefix);
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == node.children.len() - 1;
        print_tree(child, &full_prefix, is_last);
    }
}

fn format_node_label(node: &UiNode) -> String {
    let mut parts = vec![node.control_type.clone()];

    if !node.name.is_empty() {
        parts.push(format!("\"{}\"", node.name));
    }
    if !node.automation_id.is_empty() {
        parts.push(format!("id={}", node.automation_id));
    }
    if let Some(val) = &node.value {
        parts.push(format!("value={:?}", val));
    }
    if !node.enabled {
        parts.push("disabled".to_string());
    }
    if node.focused {
        parts.push("FOCUSED".to_string());
    }
    if let Some(r) = &node.rect {
        parts.push(format!("[{},{} {}x{}]", r.x, r.y, r.width, r.height));
    }

    parts.join(" ")
}

// ────────────────────────────────────────────────────────────────
// snapshot
// ────────────────────────────────────────────────────────────────

pub fn run_snapshot(
    args: &SnapshotArgs,
    manager: &dyn WindowManager,
    automation: &dyn UiAutomation,
) -> Result<()> {
    let window = manager.find_window(&args.window.resolve()?)?;
    let tree = automation.get_ui_tree(window.hwnd, Some(args.max_depth))?;

    // YAML コメントとしてヘッダを出力（パーサには影響しない）
    println!("# snapshot: \"{}\" (HWND: {})", window.title, window.hwnd);

    let mut counter = 0usize;
    print_snapshot_yaml(&tree, "", &mut counter);

    // 使い方のヒントは stderr に出力して YAML を汚染しない
    eprintln!("# click-element --hwnd {} --ref eN", window.hwnd);

    Ok(())
}

/// playwright-cli の ARIA snapshot に倣った YAML 形式で要素を出力する
fn print_snapshot_yaml(node: &UiNode, indent: &str, counter: &mut usize) {
    // 有効な矩形を持つ要素にのみ ref を割り当てる
    let ref_str = if has_visible_rect(node) {
        *counter += 1;
        format!(" [ref=e{}]", counter)
    } else {
        String::new()
    };

    let role = node.control_type.to_lowercase();
    let name_str = if node.name.is_empty() {
        String::new()
    } else {
        format!(" {:?}", node.name)
    };
    let state_str = if !node.enabled { " [disabled]" } else { "" };
    let focus_str = if node.focused { " [focused]" } else { "" };

    let has_children = !node.children.is_empty();
    let has_value = node.value.is_some();
    let colon = if has_children || has_value { ":" } else { "" };

    println!(
        "{}- {}{}{}{}{}{}",
        indent, role, name_str, state_str, focus_str, ref_str, colon
    );

    let child_indent = format!("{}  ", indent);

    if let Some(val) = &node.value {
        println!("{}  - value: {:?}", indent, val);
    }

    for child in &node.children {
        print_snapshot_yaml(child, &child_indent, counter);
    }
}

fn has_visible_rect(node: &UiNode) -> bool {
    node.rect
        .as_ref()
        .map(|r| r.width > 0 && r.height > 0)
        .unwrap_or(false)
}

/// スナップショット ref番号から UiNode を取得する（DFS で N 番目の可視要素）
pub fn find_node_by_ref(tree: &UiNode, ref_num: usize) -> Option<UiNode> {
    let mut counter = 0usize;
    find_node_recursive(tree, ref_num, &mut counter)
}

fn find_node_recursive(node: &UiNode, target: usize, counter: &mut usize) -> Option<UiNode> {
    if has_visible_rect(node) {
        *counter += 1;
        if *counter == target {
            return Some(node.clone());
        }
    }
    for child in &node.children {
        if let Some(found) = find_node_recursive(child, target, counter) {
            return Some(found);
        }
    }
    None
}

/// ref 文字列（例: "e5"）から番号を解析する
pub fn parse_ref_str(s: &str) -> Option<usize> {
    s.strip_prefix('e').and_then(|n| n.parse::<usize>().ok())
}
