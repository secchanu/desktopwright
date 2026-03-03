use serde::{Deserialize, Serialize};

/// ウィンドウを識別するための情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Windows固有: ウィンドウハンドル（十進数）
    pub hwnd: usize,
    /// ウィンドウタイトル
    pub title: String,
    /// プロセスID
    pub pid: u32,
    /// プロセス名（拡張子なし）
    pub process_name: String,
    /// Windowsウィンドウクラス名
    pub class_name: String,
    /// ウィンドウが表示されているか
    pub visible: bool,
    /// 最小化されているか
    pub minimized: bool,
    /// ウィンドウの矩形（スクリーン座標）
    pub rect: Rect,
}

/// 矩形領域
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// ウィンドウの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Minimize,
    Maximize,
    Restore,
}

/// ターゲット指定方法
#[derive(Debug, Clone)]
pub enum WindowTarget {
    /// HWND直接指定（例: "0x1A2B3C" または十進数）
    Hwnd(usize),
    /// タイトルの部分一致
    Title(String),
    /// プロセス名の部分一致
    ProcessName(String),
    /// タイトルとプロセス名の両方で絞り込み
    TitleAndProcess { title: String, process: String },
}

/// クリックボタンの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// スクロール方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 座標系の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordMode {
    /// スクリーン絶対座標
    Screen,
    /// ウィンドウ相対座標（クライアント領域の左上を原点とする）
    Window,
}

/// キャプチャオプション
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// 部分領域（ウィンドウ相対座標）
    pub region: Option<Rect>,
    /// 最大幅
    pub max_width: Option<u32>,
    /// 最大高さ
    pub max_height: Option<u32>,
    /// フォーマット
    pub format: ImageFormat,
    /// カーソルをキャプチャ画像に赤いクロスヘアで描画する
    pub cursor: bool,
}

/// 画像フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    Bmp,
}

/// UI要素のノード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiNode {
    /// コントロールタイプ（Button, Edit, Text等）
    pub control_type: String,
    /// アクセシブルな名前
    pub name: String,
    /// クラス名
    pub class_name: String,
    /// 自動化ID
    pub automation_id: String,
    /// 有効かどうか
    pub enabled: bool,
    /// キーボードフォーカスを持つか
    pub focused: bool,
    /// テキスト値（テキストコントロールの場合）
    pub value: Option<String>,
    /// 画面上の矩形
    pub rect: Option<Rect>,
    /// 子要素
    pub children: Vec<UiNode>,
}

/// click-element コマンドのテキストマッチモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMatchMode {
    /// 部分一致（デフォルト）
    Contains,
    /// 完全一致
    Exact,
}

/// click-element コマンドの結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickElementResult {
    /// マッチした要素の名前
    pub name: String,
    /// コントロールタイプ（button, edit 等）
    pub role: String,
    /// クラス名
    pub class_name: String,
    /// 要素の矩形（スクリーン座標）
    pub rect: Rect,
    /// クリックしたX座標
    pub click_x: i32,
    /// クリックしたY座標
    pub click_y: i32,
}

/// 差分検出の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// 変化が検出されたか
    pub changed: bool,
    /// 変化があった矩形領域（スクリーン座標）
    pub changed_regions: Vec<Rect>,
    /// 変化をまとめたバウンディングボックス
    pub bounding_box: Option<Rect>,
}
