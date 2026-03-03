use anyhow::Result;
use image::DynamicImage;

use crate::core::types::{
    CaptureOptions, CoordMode, DiffResult, MouseButton, ScrollDirection, TextMatchMode, UiNode,
    WindowInfo, WindowState, WindowTarget,
};

/// プラットフォーム非依存のウィンドウ操作インターフェース
pub trait WindowManager {
    /// 表示中のウィンドウ一覧を返す
    fn list_windows(&self) -> Result<Vec<WindowInfo>>;

    /// ターゲット条件に一致するウィンドウを返す（複数一致でエラー）
    fn find_window(&self, target: &WindowTarget) -> Result<WindowInfo>;

    /// ターゲット条件に一致するウィンドウを全て返す
    fn find_windows(&self, target: &WindowTarget) -> Result<Vec<WindowInfo>>;

    /// ウィンドウをフォアグラウンドに移動する
    fn focus_window(&self, hwnd: usize) -> Result<()>;

    /// ウィンドウの状態を変更する（最小化・最大化・リストア）
    fn set_window_state(&self, hwnd: usize, state: WindowState) -> Result<()>;

    /// ウィンドウのサイズを変更する（位置は変更しない）
    fn resize_window(&self, hwnd: usize, width: u32, height: u32) -> Result<()>;

    /// 現在フォアグラウンドにあるウィンドウを返す
    fn get_foreground_window(&self) -> Result<Option<WindowInfo>>;

    /// ウィンドウを閉じる（WM_CLOSE を送信する）
    fn close_window(&self, hwnd: usize) -> Result<()>;
}

/// プラットフォーム非依存のスクリーンキャプチャインターフェース
pub trait ScreenCapture {
    /// 指定ウィンドウをキャプチャする
    fn capture_window(&self, hwnd: usize, options: &CaptureOptions) -> Result<DynamicImage>;

    /// 差分検出付きで連続キャプチャを行う
    ///
    /// タイムアウトまでに変化が検出された場合は変化領域の画像を返す。
    /// 変化がなかった場合は Ok(None) を返す。
    fn capture_diff(
        &self,
        hwnd: usize,
        timeout_ms: u64,
        threshold: f32,
        options: &CaptureOptions,
    ) -> Result<Option<(DynamicImage, DiffResult)>>;
}

/// プラットフォーム非依存のマウス・キーボード入力インターフェース
pub trait InputController {
    /// マウスカーソルを移動する
    fn mouse_move(&self, x: i32, y: i32, hwnd: Option<usize>, mode: CoordMode) -> Result<()>;

    /// マウスクリックを実行する
    fn mouse_click(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        double_click: bool,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()>;

    /// マウスボタンを押したままにする（drag開始などに使用）
    fn mouse_button_down(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()>;

    /// マウスボタンを離す（drag終了などに使用）
    fn mouse_button_up(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()>;

    /// マウスホイールをスクロールする
    fn mouse_scroll(
        &self,
        direction: ScrollDirection,
        amount: i32,
        x: Option<i32>,
        y: Option<i32>,
    ) -> Result<()>;

    /// キーを送信する（例: "ctrl+c", "enter", "f5"）
    fn send_key(&self, key: &str) -> Result<()>;

    /// キーを押したままにする（修飾キーの保持などに使用）
    fn send_key_down(&self, key: &str) -> Result<()>;

    /// キーを離す（send_key_down で押したキーの解放）
    fn send_key_up(&self, key: &str) -> Result<()>;

    /// テキストを入力する（クリップボード貼り付け方式）
    fn type_text(&self, text: &str) -> Result<()>;

    /// WM_LBUTTONDOWN/UP を直接 HWND のメッセージキューに注入するクリック。
    ///
    /// SendInput はフォアグラウンドウィンドウに依存するが、PostMessage は
    /// ウィンドウのアクティブ状態・Z-order に関係なく直接メッセージを届ける。
    /// Per-Monitor DPI-Aware ウィンドウ（gpui 等）への座標は物理クライアント座標を使う。
    fn post_message_click(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: usize,
        mode: CoordMode,
    ) -> Result<()>;
}

/// プラットフォーム非依存のUI Automationインターフェース
pub trait UiAutomation {
    /// 指定ウィンドウのUI要素ツリーを取得する
    fn get_ui_tree(&self, hwnd: usize, max_depth: Option<u32>) -> Result<UiNode>;

    /// 現在フォーカスされている要素を取得する
    fn get_focused_element(&self) -> Result<Option<UiNode>>;

    /// テキストとロールで要素を検索する
    ///
    /// timeout_ms 以内に index 番目のマッチが見つかれば Some(UiNode) を返す。
    /// タイムアウトした場合は Ok(None) を返す。
    fn find_element(
        &self,
        hwnd: usize,
        text: &str,
        role: Option<&str>,
        index: usize,
        match_mode: TextMatchMode,
        timeout_ms: u64,
    ) -> Result<Option<UiNode>>;

    /// チェックボックス等のトグル要素の状態を設定する
    ///
    /// desired_state=true でチェック済み（On）に、false で未チェック（Off）に設定する。
    /// TogglePattern が利用できない要素にはエラーを返す。
    fn toggle_element(
        &self,
        hwnd: usize,
        text: &str,
        role: Option<&str>,
        desired_state: bool,
        timeout_ms: u64,
    ) -> Result<()>;

    /// コンボボックス・リストボックスのオプションを選択する
    ///
    /// element_text が Some の場合はその名前のコンボボックスを展開してから選択する。
    /// None の場合は option_text に一致する選択肢を直接探して選択する。
    fn select_option(
        &self,
        hwnd: usize,
        element_text: Option<&str>,
        option_text: &str,
        timeout_ms: u64,
    ) -> Result<()>;
}

/// 現在のプラットフォームのインターフェース一式
pub struct Platform {
    pub window_manager: Box<dyn WindowManager>,
    pub screen_capture: Box<dyn ScreenCapture>,
    pub input_controller: Box<dyn InputController>,
    pub ui_automation: Box<dyn UiAutomation>,
}
