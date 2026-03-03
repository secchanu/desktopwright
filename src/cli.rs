use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::core::types::{CoordMode, MouseButton, WindowTarget};

pub mod app;
pub mod capture;
pub mod install;
pub mod click_element;
pub mod drag;
pub mod element_action;
pub mod focus;
pub mod get_text;
pub mod key;
pub mod list;
pub mod mouse;
pub mod scroll;
pub mod type_text;
pub mod ui_tree;
pub mod wait;
pub mod window;

/// HWND文字列（0x前置または十進数）を usize に変換する
pub fn parse_hwnd(s: &str) -> Result<usize> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(hex, 16).map_err(|e| anyhow::anyhow!("HWND解析失敗: {}", e))
    } else {
        s.parse::<usize>().map_err(|e| anyhow::anyhow!("HWND解析失敗: {}", e))
    }
}

/// --target / --process / --hwnd の共通ウィンドウ指定引数
#[derive(Args, Debug, Clone)]
pub struct WindowTargetArgs {
    /// ターゲットウィンドウ（タイトル部分一致）
    #[arg(long, short = 't')]
    pub target: Option<String>,

    /// プロセス名でターゲット指定（部分一致）
    #[arg(long)]
    pub process: Option<String>,

    /// HWND直接指定（十進数または0x前置の十六進数）
    #[arg(long)]
    pub hwnd: Option<String>,
}

impl WindowTargetArgs {
    pub fn resolve(&self) -> Result<WindowTarget> {
        if let Some(s) = &self.hwnd {
            return Ok(WindowTarget::Hwnd(parse_hwnd(s)?));
        }
        match (&self.target, &self.process) {
            (Some(t), Some(p)) => Ok(WindowTarget::TitleAndProcess {
                title: t.clone(),
                process: p.clone(),
            }),
            (Some(t), None) => Ok(WindowTarget::Title(t.clone())),
            (None, Some(p)) => Ok(WindowTarget::ProcessName(p.clone())),
            (None, None) => Err(anyhow::anyhow!(
                "--target、--process、または --hwnd のいずれかを指定してください"
            )),
        }
    }
}

/// マウスボタン選択（click / drag / mousedown / mouseup コマンドで共用）
#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum ButtonArg {
    Left,
    Right,
    Middle,
}

impl From<ButtonArg> for MouseButton {
    fn from(b: ButtonArg) -> Self {
        match b {
            ButtonArg::Left => MouseButton::Left,
            ButtonArg::Right => MouseButton::Right,
            ButtonArg::Middle => MouseButton::Middle,
        }
    }
}

/// 座標系選択（click / move / drag / mousedown / mouseup コマンドで共用）
#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum CoordModeArg {
    Screen,
    Window,
}

impl From<CoordModeArg> for CoordMode {
    fn from(c: CoordModeArg) -> Self {
        match c {
            CoordModeArg::Screen => CoordMode::Screen,
            CoordModeArg::Window => CoordMode::Window,
        }
    }
}

/// Windows GUI自動化CLIツール
///
/// Playwright CLIの操作パターンを参考にしたコマンド体系。
/// AIエージェントが自律的に操作を行う際の使用を想定している。
#[derive(Parser, Debug)]
#[command(name = "desktopwright", author, version, about, long_about = None)]
pub struct Cli {
    /// JSON形式で出力する（全コマンドで使用可能）
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 実行中ウィンドウの一覧を表示する
    List(list::ListArgs),

    /// 特定ウィンドウをキャプチャする
    Capture(capture::CaptureArgs),

    /// ウィンドウをフォアグラウンドに移動する
    Focus(focus::FocusArgs),

    /// ウィンドウの状態を変更する（最小化・最大化・リストア）
    Window(window::WindowArgs),

    /// ウィンドウのサイズを変更する
    Resize(window::ResizeArgs),

    /// マウスクリックを実行する
    Click(mouse::ClickArgs),

    /// マウスカーソルを移動する
    #[command(alias = "hover")]
    Move(mouse::MoveArgs),

    /// マウスドラッグを実行する（mousedown → move → mouseup）
    Drag(drag::DragArgs),

    /// マウスボタンを押したままにする
    #[command(name = "mousedown")]
    Mousedown(drag::MousedownArgs),

    /// マウスボタンを離す
    #[command(name = "mouseup")]
    Mouseup(drag::MouseupArgs),

    /// マウスホイールをスクロールする
    Scroll(scroll::ScrollArgs),

    /// キーを送信する（例: ctrl+c, enter, f5）
    #[command(alias = "press")]
    Key(key::KeyArgs),

    /// キーを押したままにする（修飾キーの保持に使用）
    #[command(name = "keydown")]
    Keydown(key::KeydownArgs),

    /// キーを離す（keydown で押したキーに対応）
    #[command(name = "keyup")]
    Keyup(key::KeyupArgs),

    /// テキストを入力する（クリップボード貼り付け方式）
    Type(type_text::TypeArgs),

    /// ウィンドウのUI要素ツリーを表示する
    #[command(name = "ui-tree")]
    UiTree(ui_tree::UiTreeArgs),

    /// アクセシビリティスナップショットを表示する（playwright-cli の snapshot 相当）
    Snapshot(ui_tree::SnapshotArgs),

    /// 現在フォアグラウンドのウィンドウを表示する
    Foreground(list::ForegroundArgs),

    /// テキストまたはロールでUI要素を検索してクリックする
    #[command(name = "click-element")]
    ClickElement(click_element::ClickElementArgs),

    /// チェックボックスをチェック状態にする
    Check(element_action::CheckArgs),

    /// チェックボックスのチェックを外す
    Uncheck(element_action::UncheckArgs),

    /// コンボボックス・リストボックスのオプションを選択する
    Select(element_action::SelectArgs),

    /// ダイアログを承認する（Enter キーを送信）
    #[command(name = "dialog-accept")]
    DialogAccept(element_action::DialogAcceptArgs),

    /// ダイアログを閉じる（Escape キーを送信）
    #[command(name = "dialog-dismiss")]
    DialogDismiss(element_action::DialogDismissArgs),

    /// 指定ミリ秒だけ待機する
    Wait(wait::WaitArgs),

    /// ウィンドウが現れるまで待機する（アプリ起動後の待機に使用）
    #[command(name = "wait-for-window")]
    WaitForWindow(wait::WaitForWindowArgs),

    /// アプリを起動する
    Launch(app::LaunchArgs),

    /// ウィンドウを閉じる（WM_CLOSE を送信）
    Close(app::CloseArgs),

    /// UI要素のテキスト値を取得する（アサーション・確認に使用）
    #[command(name = "get-text")]
    GetText(get_text::GetTextArgs),

    /// AIエージェント統合スキルをインストールする（例: desktopwright install --skills）
    Install(install::InstallArgs),
}
