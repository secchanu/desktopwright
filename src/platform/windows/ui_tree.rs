use anyhow::{Result, anyhow};
use uiautomation::patterns::{UIExpandCollapsePattern, UISelectionItemPattern, UITogglePattern};
use uiautomation::types::{Handle, ToggleState};
use uiautomation::{UIAutomation, UIElement, UITreeWalker};

use crate::core::platform::UiAutomation as UiAutomationTrait;
use crate::core::types::{Rect, TextMatchMode, UiNode};

pub struct WindowsUiAutomation;

impl WindowsUiAutomation {
    pub fn new() -> Self {
        WindowsUiAutomation
    }

    /// UIAutomationを初期化してルート要素とウォーカーを返す
    ///
    /// ポーリングループ内で毎回呼ぶことで、UIキャッシュを使わず最新状態を取得する。
    fn get_root_and_walker(hwnd: usize) -> Result<(UIElement, UITreeWalker)> {
        let automation =
            UIAutomation::new().map_err(|e| anyhow!("UI Automation初期化失敗: {}", e))?;
        let handle = Handle::from(hwnd as isize);
        let root = automation
            .element_from_handle(handle)
            .map_err(|e| anyhow!("ウィンドウ要素取得失敗: {}", e))?;
        let walker = automation
            .get_control_view_walker()
            .map_err(|e| anyhow!("TreeWalker取得失敗: {}", e))?;
        Ok((root, walker))
    }

    /// UIElementを再帰的にUiNodeに変換する
    fn element_to_node(
        element: &UIElement,
        walker: &UITreeWalker,
        depth: u32,
        max_depth: u32,
    ) -> UiNode {
        let control_type = element
            .get_control_type()
            .map(|ct| format!("{:?}", ct))
            .unwrap_or_else(|_| "Unknown".to_string());

        let name = element.get_name().unwrap_or_default();
        let class_name = element.get_classname().unwrap_or_default();
        let automation_id = element.get_automation_id().unwrap_or_default();

        let enabled = element.is_enabled().unwrap_or(false);
        let focused = element.has_keyboard_focus().unwrap_or(false);

        // テキスト値の取得（Valueパターン対応要素のみ）
        let value = element
            .get_property_value(uiautomation::types::UIProperty::ValueValue)
            .ok()
            .and_then(|v| v.get_string().ok())
            .filter(|s| !s.is_empty());

        // 画面上の矩形
        let rect = element.get_bounding_rectangle().ok().map(|r| Rect {
            x: r.get_left(),
            y: r.get_top(),
            width: r.get_width(),
            height: r.get_height(),
        });

        let children = if depth < max_depth {
            Self::get_children(element, walker, depth + 1, max_depth)
        } else {
            vec![]
        };

        UiNode {
            control_type,
            name,
            class_name,
            automation_id,
            enabled,
            focused,
            value,
            rect,
            children,
        }
    }

    /// UI要素ツリーをDFSで走査し、各要素に visitor を適用する（深さ上限 30）
    fn dfs_walk<F>(element: &UIElement, walker: &UITreeWalker, depth: u32, visitor: &mut F)
    where
        F: FnMut(&UIElement),
    {
        if depth > 30 {
            return;
        }
        visitor(element);
        let first = match walker.get_first_child(element) {
            Ok(c) => c,
            Err(_) => return,
        };
        Self::dfs_walk(&first, walker, depth + 1, visitor);
        let mut current = first;
        loop {
            let sibling = walker.get_next_sibling(&current);
            match sibling {
                Ok(next) => {
                    Self::dfs_walk(&next, walker, depth + 1, visitor);
                    current = next;
                }
                Err(_) => break,
            }
        }
    }

    /// テキストとロールのフィルタ条件を満たす場合にアクセシブル名を返す（collect_matching 系で共用）
    ///
    /// Some(name) を返すことで呼び出し側での get_name() の二重呼び出しを防ぐ。
    fn element_matches(
        element: &UIElement,
        text_lower: &str,
        role_lower: Option<&str>,
        match_mode: TextMatchMode,
    ) -> Option<String> {
        let name = element.get_name().unwrap_or_default();
        if name.is_empty() {
            return None;
        }
        let name_lower = name.to_lowercase();
        let text_matched = match match_mode {
            TextMatchMode::Contains => name_lower.contains(text_lower),
            TextMatchMode::Exact => name_lower == text_lower,
        };
        if !text_matched {
            return None;
        }
        if let Some(r) = role_lower {
            let control_type = element
                .get_control_type()
                .map(|ct| format!("{:?}", ct).to_lowercase())
                .unwrap_or_default();
            if control_type != r {
                return None;
            }
        }
        Some(name)
    }

    /// テキスト・ロールでマッチする要素を UiNode として収集する
    fn collect_matching(
        root: &UIElement,
        walker: &UITreeWalker,
        text_lower: &str,
        role_lower: Option<&str>,
        match_mode: TextMatchMode,
        results: &mut Vec<UiNode>,
    ) {
        Self::dfs_walk(root, walker, 0, &mut |element| {
            let Some(name) = Self::element_matches(element, text_lower, role_lower, match_mode)
            else {
                return;
            };
            let rect = element.get_bounding_rectangle().ok().and_then(|r| {
                let w = r.get_width();
                let h = r.get_height();
                // 矩形が有効な要素のみ収集する
                if w > 0 && h > 0 {
                    Some(Rect {
                        x: r.get_left(),
                        y: r.get_top(),
                        width: w,
                        height: h,
                    })
                } else {
                    None
                }
            });
            if let Some(rect) = rect {
                let control_type = element
                    .get_control_type()
                    .map(|ct| format!("{:?}", ct).to_lowercase())
                    .unwrap_or_default();
                results.push(UiNode {
                    control_type,
                    name,
                    class_name: element.get_classname().unwrap_or_default(),
                    automation_id: element.get_automation_id().unwrap_or_default(),
                    enabled: element.is_enabled().unwrap_or(false),
                    focused: element.has_keyboard_focus().unwrap_or(false),
                    value: None,
                    rect: Some(rect),
                    children: vec![],
                });
            }
        });
    }

    /// テキスト・ロールでマッチする要素を UIElement として収集する（パターン操作に使用）
    fn collect_matching_elements(
        root: &UIElement,
        walker: &UITreeWalker,
        text_lower: &str,
        role_lower: Option<&str>,
        match_mode: TextMatchMode,
        results: &mut Vec<UIElement>,
    ) {
        Self::dfs_walk(root, walker, 0, &mut |element| {
            if Self::element_matches(element, text_lower, role_lower, match_mode).is_some() {
                results.push(element.clone());
            }
        });
    }

    fn get_children(
        element: &UIElement,
        walker: &UITreeWalker,
        depth: u32,
        max_depth: u32,
    ) -> Vec<UiNode> {
        let mut children = Vec::new();

        let first = match walker.get_first_child(element) {
            Ok(c) => c,
            Err(_) => return children,
        };

        children.push(Self::element_to_node(&first, walker, depth, max_depth));

        let mut current = first;
        loop {
            let sibling = walker.get_next_sibling(&current);
            match sibling {
                Ok(next) => {
                    children.push(Self::element_to_node(&next, walker, depth, max_depth));
                    current = next;
                }
                Err(_) => break,
            }
        }

        children
    }
}

impl UiAutomationTrait for WindowsUiAutomation {
    fn get_ui_tree(&self, hwnd: usize, max_depth: Option<u32>) -> Result<UiNode> {
        let (element, walker) = Self::get_root_and_walker(hwnd)?;
        let depth_limit = max_depth.unwrap_or(5);
        Ok(Self::element_to_node(&element, &walker, 0, depth_limit))
    }

    fn get_focused_element(&self) -> Result<Option<UiNode>> {
        let automation =
            UIAutomation::new().map_err(|e| anyhow!("UI Automation初期化失敗: {}", e))?;
        let walker = automation
            .get_control_view_walker()
            .map_err(|e| anyhow!("TreeWalker取得失敗: {}", e))?;
        let focused = automation.get_focused_element();
        match focused {
            Ok(element) => Ok(Some(Self::element_to_node(&element, &walker, 0, 0))),
            Err(_) => Ok(None),
        }
    }

    fn toggle_element(
        &self,
        hwnd: usize,
        text: &str,
        role: Option<&str>,
        desired_state: bool,
        timeout_ms: u64,
    ) -> Result<()> {
        let text_lower = text.to_lowercase();
        let role_lower = role.map(|r| r.to_lowercase());
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        loop {
            let (root, walker) = Self::get_root_and_walker(hwnd)?;
            let mut elements: Vec<UIElement> = Vec::new();
            Self::collect_matching_elements(
                &root,
                &walker,
                &text_lower,
                role_lower.as_deref(),
                TextMatchMode::Contains,
                &mut elements,
            );

            if let Some(element) = elements.first() {
                let toggle = element
                    .get_pattern::<UITogglePattern>()
                    .map_err(|e| anyhow!("TogglePatternが利用できません: {}", e))?;
                let current_state = toggle
                    .get_toggle_state()
                    .map_err(|e| anyhow!("トグル状態の取得失敗: {}", e))?;
                if (current_state == ToggleState::On) != desired_state {
                    toggle
                        .toggle()
                        .map_err(|e| anyhow!("トグル操作失敗: {}", e))?;
                }
                return Ok(());
            }

            if std::time::Instant::now() >= deadline {
                return Err(anyhow!(
                    "要素が見つかりません: text={:?}, role={:?}",
                    text,
                    role
                ));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn select_option(
        &self,
        hwnd: usize,
        element_text: Option<&str>,
        option_text: &str,
        timeout_ms: u64,
    ) -> Result<()> {
        let option_lower = option_text.to_lowercase();
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        loop {
            let (root, walker) = Self::get_root_and_walker(hwnd)?;

            // コンボボックスのテキストが指定された場合は展開を試みる
            if let Some(combo_text) = element_text {
                let combo_lower = combo_text.to_lowercase();
                let mut combos: Vec<UIElement> = Vec::new();
                Self::collect_matching_elements(
                    &root,
                    &walker,
                    &combo_lower,
                    Some("combobox"),
                    TextMatchMode::Contains,
                    &mut combos,
                );
                if let Some(combo) = combos.first() {
                    let expand_result = combo.get_pattern::<UIExpandCollapsePattern>();
                    if let Ok(expand) = expand_result {
                        let _ = expand.expand();
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                }
            }

            // 選択肢を検索して SelectionItemPattern で選択する
            let mut items: Vec<UIElement> = Vec::new();
            Self::collect_matching_elements(
                &root,
                &walker,
                &option_lower,
                None,
                TextMatchMode::Contains,
                &mut items,
            );

            for item in &items {
                let select_result = item.get_pattern::<UISelectionItemPattern>();
                if let Ok(select) = select_result {
                    select
                        .select()
                        .map_err(|e| anyhow!("選択操作失敗: {}", e))?;
                    return Ok(());
                }
            }

            if std::time::Instant::now() >= deadline {
                return Err(anyhow!("選択肢が見つかりません: {:?}", option_text));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn find_element(
        &self,
        hwnd: usize,
        text: &str,
        role: Option<&str>,
        index: usize,
        match_mode: TextMatchMode,
        timeout_ms: u64,
    ) -> Result<Option<UiNode>> {
        let text_lower = text.to_lowercase();
        let role_lower = role.map(|r| r.to_lowercase());
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        loop {
            let (root, walker) = Self::get_root_and_walker(hwnd)?;
            let mut results = Vec::new();
            Self::collect_matching(
                &root,
                &walker,
                &text_lower,
                role_lower.as_deref(),
                match_mode,
                &mut results,
            );

            if results.len() > index {
                return Ok(Some(results.remove(index)));
            }

            if std::time::Instant::now() >= deadline {
                return Ok(None);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
