use anyhow::{Result, anyhow};
use windows::Win32::Foundation::{HANDLE, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{ClientToScreen, ScreenToClient};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetDoubleClickTime, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    MapVirtualKeyW, SendInput, VIRTUAL_KEY, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN,
    VK_END, VK_ESCAPE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10,
    VK_F11, VK_F12, VK_HOME, VK_INSERT, VK_LEFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN,
    VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, PostMessageW, SM_CXSCREEN, SM_CYSCREEN, SetForegroundWindow, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
};

use crate::core::platform::InputController;
use crate::core::types::{CoordMode, MouseButton, ScrollDirection};

use super::to_hwnd;

pub struct WindowsInputController;

impl WindowsInputController {
    pub fn new() -> Self {
        WindowsInputController
    }

    /// キー名をVIRTUAL_KEYに変換する
    fn key_name_to_vk(name: &str) -> Option<VIRTUAL_KEY> {
        match name.to_lowercase().as_str() {
            "enter" | "return" => Some(VK_RETURN),
            "tab" => Some(VK_TAB),
            "space" => Some(VK_SPACE),
            "backspace" | "back" => Some(VK_BACK),
            "delete" | "del" => Some(VK_DELETE),
            "insert" | "ins" => Some(VK_INSERT),
            "escape" | "esc" => Some(VK_ESCAPE),
            "home" => Some(VK_HOME),
            "end" => Some(VK_END),
            "pageup" | "pgup" => Some(VK_PRIOR),
            "pagedown" | "pgdn" => Some(VK_NEXT),
            "left" => Some(VK_LEFT),
            "right" => Some(VK_RIGHT),
            "up" => Some(VK_UP),
            "down" => Some(VK_DOWN),
            "f1" => Some(VK_F1),
            "f2" => Some(VK_F2),
            "f3" => Some(VK_F3),
            "f4" => Some(VK_F4),
            "f5" => Some(VK_F5),
            "f6" => Some(VK_F6),
            "f7" => Some(VK_F7),
            "f8" => Some(VK_F8),
            "f9" => Some(VK_F9),
            "f10" => Some(VK_F10),
            "f11" => Some(VK_F11),
            "f12" => Some(VK_F12),
            "ctrl" | "control" => Some(VK_CONTROL),
            "shift" => Some(VK_SHIFT),
            "alt" => Some(VK_MENU),
            "win" | "windows" | "super" => Some(VK_LWIN),
            "capslock" | "caps" => Some(VK_CAPITAL),
            _ => {
                // 単一文字の場合はVkKeyScanで変換
                if name.len() == 1 {
                    let c = name.chars().next().unwrap();
                    let vk = unsafe {
                        windows::Win32::UI::Input::KeyboardAndMouse::VkKeyScanW(c as u16)
                    };
                    if vk != -1 {
                        Some(VIRTUAL_KEY((vk & 0xFF) as u16))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// キーコンボ文字列（例: "ctrl+c"）をパースして修飾キーと主キーに分解する
    fn parse_key_combo(key_str: &str) -> Result<(Vec<VIRTUAL_KEY>, VIRTUAL_KEY)> {
        let parts: Vec<&str> = key_str.split('+').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return Err(anyhow!("キー文字列が空です"));
        }

        let main_key_str = parts.last().unwrap();
        let modifier_strs = &parts[..parts.len() - 1];

        let main_vk = Self::key_name_to_vk(main_key_str)
            .ok_or_else(|| anyhow!("不明なキー: '{}'", main_key_str))?;

        let modifiers: Result<Vec<VIRTUAL_KEY>> = modifier_strs
            .iter()
            .map(|s| Self::key_name_to_vk(s).ok_or_else(|| anyhow!("不明な修飾キー: '{}'", s)))
            .collect();

        Ok((modifiers?, main_vk))
    }

    /// INPUT構造体を作成してキーを押す
    fn make_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
        let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) } as u16;
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: if key_up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS(0)
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    /// スクリーン座標に変換する（CoordMode::Windowの場合はHWND基準でClientToScreenを呼ぶ）
    fn to_screen_coords(
        x: i32,
        y: i32,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<(i32, i32)> {
        match mode {
            CoordMode::Screen => Ok((x, y)),
            CoordMode::Window => {
                let h = hwnd.ok_or_else(|| anyhow!("ウィンドウ相対座標にはHWNDが必要です"))?;
                let hwnd = to_hwnd(h);
                let mut pt = POINT { x, y };
                unsafe {
                    let _ = ClientToScreen(hwnd, &mut pt);
                }
                Ok((pt.x, pt.y))
            }
        }
    }

    /// HWND が指定されている場合、SendInput 前に対象ウィンドウをフォアグラウンドに移動する。
    ///
    /// gpui 等のフレームワークは非アクティブウィンドウへの WM_MOUSEACTIVATE を
    /// MA_ACTIVATEANDEAT で処理するため、click コマンドが別プロセスで実行されて
    /// ターミナルがフォアグラウンドを奪い返した後にクリックが無視される問題を防ぐ。
    /// 同一プロセス内で SetForegroundWindow を呼ぶことで race condition を回避する。
    fn activate_window(hwnd: Option<usize>) {
        let Some(h) = hwnd else { return };
        let hwnd_win32 = to_hwnd(h);
        unsafe {
            let _ = SetForegroundWindow(hwnd_win32);
        }
        // ウィンドウが WM_ACTIVATE を処理するまで待つ
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    fn send_inputs(inputs: &[INPUT]) -> Result<()> {
        let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent != inputs.len() as u32 {
            return Err(anyhow!("SendInput: {}件中{}件のみ送信", inputs.len(), sent));
        }
        Ok(())
    }

    /// マウスを絶対座標に移動するINPUTを作成する
    ///
    /// 座標系: `SetProcessDPIAware` により物理ピクセル座標を使用する。
    /// xcap によるキャプチャ画像の座標をそのまま click --coord window に渡せる。
    /// ただし DWM 拡張フレーム境界（ウィンドウシャドウを含む）分のオフセットが
    /// xcap クライアント原点とズレる場合がある点に注意。
    fn make_mouse_move_input(x: i32, y: i32) -> INPUT {
        let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let norm_x = (x * 65535 / screen_w.max(1)) as i32;
        let norm_y = (y * 65535 / screen_h.max(1)) as i32;

        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: norm_x,
                    dy: norm_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    /// マウスボタンイベント（押す・離す）のINPUTを作成する
    fn make_mouse_button_input(flag: MOUSE_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    /// クリップボードにUTF-16文字列を設定する（CF_UNICODETEXT）
    fn set_clipboard_text(text: &str) -> Result<()> {
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = utf16.len() * 2;

        unsafe {
            OpenClipboard(None).map_err(|e| anyhow!("OpenClipboard失敗: {}", e))?;
            EmptyClipboard().map_err(|e| {
                let _ = CloseClipboard();
                anyhow!("EmptyClipboard失敗: {}", e)
            })?;

            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len).map_err(|e| {
                let _ = CloseClipboard();
                anyhow!("GlobalAlloc失敗: {}", e)
            })?;

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                let _ = CloseClipboard();
                return Err(anyhow!("GlobalLock失敗"));
            }
            std::ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr as *mut u8, byte_len);
            let _ = GlobalUnlock(hmem);

            // HGLOBALをHANDLEに変換してSetClipboardDataに渡す（CF_UNICODETEXT = 13）
            let handle = HANDLE(hmem.0);
            SetClipboardData(13, Some(handle)).map_err(|e| {
                let _ = CloseClipboard();
                anyhow!("SetClipboardData失敗: {}", e)
            })?;

            CloseClipboard().map_err(|e| anyhow!("CloseClipboard失敗: {}", e))?;
        }
        Ok(())
    }
}

impl InputController for WindowsInputController {
    fn mouse_move(&self, x: i32, y: i32, hwnd: Option<usize>, mode: CoordMode) -> Result<()> {
        let (sx, sy) = Self::to_screen_coords(x, y, hwnd, mode)?;
        let input = Self::make_mouse_move_input(sx, sy);
        Self::send_inputs(&[input])
    }

    fn mouse_button_down(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()> {
        Self::activate_window(hwnd);
        let (sx, sy) = Self::to_screen_coords(x, y, hwnd, mode)?;
        let move_input = Self::make_mouse_move_input(sx, sy);
        Self::send_inputs(&[move_input])?;

        let down_flag = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
            MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
        };
        Self::send_inputs(&[Self::make_mouse_button_input(down_flag)])
    }

    fn mouse_button_up(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()> {
        let (sx, sy) = Self::to_screen_coords(x, y, hwnd, mode)?;
        let move_input = Self::make_mouse_move_input(sx, sy);
        Self::send_inputs(&[move_input])?;

        let up_flag = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTUP,
            MouseButton::Right => MOUSEEVENTF_RIGHTUP,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
        };
        Self::send_inputs(&[Self::make_mouse_button_input(up_flag)])
    }

    fn mouse_click(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        double_click: bool,
        hwnd: Option<usize>,
        mode: CoordMode,
    ) -> Result<()> {
        Self::activate_window(hwnd);
        let (sx, sy) = Self::to_screen_coords(x, y, hwnd, mode)?;

        let move_input = Self::make_mouse_move_input(sx, sy);
        Self::send_inputs(&[move_input])?;

        let click_delay_ms = if double_click {
            unsafe { GetDoubleClickTime() / 2 }
        } else {
            0
        };

        let (down_flag, up_flag) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };

        Self::send_inputs(&[
            Self::make_mouse_button_input(down_flag),
            Self::make_mouse_button_input(up_flag),
        ])?;

        if double_click {
            std::thread::sleep(std::time::Duration::from_millis(click_delay_ms as u64));
            Self::send_inputs(&[
                Self::make_mouse_button_input(down_flag),
                Self::make_mouse_button_input(up_flag),
            ])?;
        }

        Ok(())
    }

    fn mouse_scroll(
        &self,
        direction: ScrollDirection,
        amount: i32,
        x: Option<i32>,
        y: Option<i32>,
    ) -> Result<()> {
        // 座標が指定された場合はそこに移動
        if let (Some(px), Some(py)) = (x, y) {
            let move_input = Self::make_mouse_move_input(px, py);
            Self::send_inputs(&[move_input])?;
        }

        // WHEEL_DELTA = 120
        let (flag, data) = match direction {
            ScrollDirection::Up => (MOUSEEVENTF_WHEEL, (120 * amount) as i32 as u32),
            ScrollDirection::Down => (MOUSEEVENTF_WHEEL, (-(120 * amount)) as i32 as u32),
            ScrollDirection::Right => (MOUSEEVENTF_HWHEEL, (120 * amount) as i32 as u32),
            ScrollDirection::Left => (MOUSEEVENTF_HWHEEL, (-(120 * amount)) as i32 as u32),
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: data,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        Self::send_inputs(&[input])
    }

    fn send_key(&self, key: &str) -> Result<()> {
        let (modifiers, main_vk) = Self::parse_key_combo(key)?;

        let mut inputs: Vec<INPUT> = Vec::new();

        // 修飾キーを押す
        for &modifier in &modifiers {
            inputs.push(Self::make_key_input(modifier, false));
        }

        // メインキーを押して離す
        inputs.push(Self::make_key_input(main_vk, false));
        inputs.push(Self::make_key_input(main_vk, true));

        // 修飾キーを逆順に離す
        for &modifier in modifiers.iter().rev() {
            inputs.push(Self::make_key_input(modifier, true));
        }

        Self::send_inputs(&inputs)
    }

    fn send_key_down(&self, key: &str) -> Result<()> {
        // 単一キーのみ受け付ける（+区切りコンボは send_key を使う）
        let vk = Self::key_name_to_vk(key).ok_or_else(|| anyhow!("不明なキー: '{}'", key))?;
        Self::send_inputs(&[Self::make_key_input(vk, false)])
    }

    fn send_key_up(&self, key: &str) -> Result<()> {
        let vk = Self::key_name_to_vk(key).ok_or_else(|| anyhow!("不明なキー: '{}'", key))?;
        Self::send_inputs(&[Self::make_key_input(vk, true)])
    }

    fn type_text(&self, text: &str) -> Result<()> {
        // クリップボード貼り付け方式でテキストを入力する。
        // IME経由での日本語入力は環境依存が大きいため、この方式を採用している。
        // 副作用として既存のクリップボード内容が上書きされる点に注意。
        Self::set_clipboard_text(text)?;

        // Ctrl+V で貼り付け
        self.send_key("ctrl+v")?;

        // 貼り付け完了を待つ
        std::thread::sleep(std::time::Duration::from_millis(50));

        Ok(())
    }

    fn post_message_click(
        &self,
        x: i32,
        y: i32,
        button: MouseButton,
        hwnd: usize,
        mode: CoordMode,
    ) -> Result<()> {
        let hwnd_win32 = to_hwnd(hwnd);

        // --coord screen の場合は ScreenToClient で物理クライアント座標に変換する。
        // gpui 等の Per-Monitor DPI-Aware ウィンドウは lparam に物理クライアント座標を期待する。
        let (client_x, client_y) = match mode {
            CoordMode::Window => (x, y),
            CoordMode::Screen => {
                let mut pt = POINT { x, y };
                unsafe {
                    let _ = ScreenToClient(hwnd_win32, &mut pt);
                }
                (pt.x, pt.y)
            }
        };

        // MAKELPARAM: loword = x, hiword = y（符号付き16bitを符号なし16bitとして格納）
        let lparam = LPARAM(
            ((client_y as i16 as u16 as u32) << 16 | (client_x as i16 as u16 as u32)) as isize,
        );

        let (down_msg, up_msg, down_wparam) = match button {
            MouseButton::Left => (WM_LBUTTONDOWN, WM_LBUTTONUP, WPARAM(0x0001)), // MK_LBUTTON
            MouseButton::Right => (WM_RBUTTONDOWN, WM_RBUTTONUP, WPARAM(0x0002)), // MK_RBUTTON
            MouseButton::Middle => (WM_MBUTTONDOWN, WM_MBUTTONUP, WPARAM(0x0010)), // MK_MBUTTON
        };

        unsafe {
            PostMessageW(Some(hwnd_win32), down_msg, down_wparam, lparam)
                .map_err(|e| anyhow!("PostMessageW(down)失敗: {}", e))?;
            PostMessageW(Some(hwnd_win32), up_msg, WPARAM(0), lparam)
                .map_err(|e| anyhow!("PostMessageW(up)失敗: {}", e))?;
        }

        Ok(())
    }
}
