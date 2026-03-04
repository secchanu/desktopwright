use anyhow::Result;
use std::path::PathBuf;
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostMessageW, SW_MAXIMIZE, SW_MINIMIZE,
    SW_RESTORE, SWP_NOMOVE, SWP_NOZORDER, SetForegroundWindow, SetWindowPos, ShowWindow, WM_CLOSE,
};
use windows::core::BOOL;

use crate::core::error::DesktopError;
use crate::core::platform::WindowManager;
use crate::core::types::{Rect, WindowInfo, WindowState, WindowTarget};

pub struct WindowsWindowManager;

use super::to_hwnd;

/// Win32 HWND型をusizeに変換する
fn from_hwnd(hwnd: HWND) -> usize {
    hwnd.0 as usize
}

impl WindowsWindowManager {
    pub fn new() -> Self {
        WindowsWindowManager
    }

    fn get_window_title(hwnd: HWND) -> String {
        let mut buf = vec![0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }

    fn get_process_info(hwnd: HWND) -> (u32, String) {
        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == 0 {
            return (0, String::new());
        }

        let process_name = unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
            match handle {
                Ok(h) => {
                    let mut buf = vec![0u16; 512];
                    let mut len = buf.len() as u32;
                    if QueryFullProcessImageNameW(
                        h,
                        PROCESS_NAME_WIN32,
                        windows::core::PWSTR(buf.as_mut_ptr()),
                        &mut len,
                    )
                    .is_ok()
                    {
                        let path = String::from_utf16_lossy(&buf[..len as usize]);
                        PathBuf::from(path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string()
                    } else {
                        String::new()
                    }
                }
                Err(_) => String::new(),
            }
        };

        (pid, process_name)
    }

    fn get_window_rect(hwnd: HWND) -> Rect {
        // DwmGetWindowAttributeでDWMが管理する実際のフレーム矩形を取得（影・枠を除く）
        // Win10以降で利用可能。失敗した場合はGetWindowRectにフォールバック。
        let mut dwm_rect = RECT::default();
        let dwm_ok = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut dwm_rect as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            )
        }
        .is_ok();

        if dwm_ok {
            Rect {
                x: dwm_rect.left,
                y: dwm_rect.top,
                width: dwm_rect.right - dwm_rect.left,
                height: dwm_rect.bottom - dwm_rect.top,
            }
        } else {
            let mut rect = RECT::default();
            unsafe {
                let _ = GetWindowRect(hwnd, &mut rect);
            };
            Rect {
                x: rect.left,
                y: rect.top,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
            }
        }
    }

    fn get_class_name(hwnd: HWND) -> String {
        let mut buf = vec![0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }

    fn hwnd_to_window_info(hwnd: HWND) -> WindowInfo {
        let title = Self::get_window_title(hwnd);
        let (pid, process_name) = Self::get_process_info(hwnd);
        let class_name = Self::get_class_name(hwnd);
        let visible = unsafe { IsWindowVisible(hwnd) }.as_bool();
        let minimized = unsafe { IsIconic(hwnd) }.as_bool();
        let rect = Self::get_window_rect(hwnd);

        WindowInfo {
            hwnd: from_hwnd(hwnd),
            title,
            pid,
            process_name,
            class_name,
            visible,
            minimized,
            rect,
        }
    }

    fn matches_target(info: &WindowInfo, target: &WindowTarget) -> bool {
        match target {
            WindowTarget::Hwnd(h) => info.hwnd == *h,
            WindowTarget::Title(t) => info.title.to_lowercase().contains(&t.to_lowercase()),
            WindowTarget::ProcessName(p) => {
                info.process_name.to_lowercase().contains(&p.to_lowercase())
            }
            WindowTarget::TitleAndProcess { title, process } => {
                info.title.to_lowercase().contains(&title.to_lowercase())
                    && info
                        .process_name
                        .to_lowercase()
                        .contains(&process.to_lowercase())
            }
        }
    }
}

// EnumWindowsのコールバック用構造体
struct EnumWindowsData {
    windows: Vec<WindowInfo>,
    visible_only: bool,
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = unsafe { &mut *(lparam.0 as *mut EnumWindowsData) };

    // 不可視ウィンドウを除外するオプション
    if data.visible_only && !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return BOOL(1);
    }

    // タイトルが空で不可視のウィンドウはスキップ（システム内部ウィンドウを除外）
    let title = WindowsWindowManager::get_window_title(hwnd);
    if title.is_empty() && !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return BOOL(1);
    }

    data.windows
        .push(WindowsWindowManager::hwnd_to_window_info(hwnd));
    BOOL(1)
}

impl WindowManager for WindowsWindowManager {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let mut data = EnumWindowsData {
            windows: Vec::new(),
            visible_only: true,
        };

        unsafe {
            EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut data as *mut _ as isize),
            )?;
        }

        Ok(data.windows)
    }

    fn find_windows(&self, target: &WindowTarget) -> Result<Vec<WindowInfo>> {
        // HWND直接指定の場合は検索不要
        if let WindowTarget::Hwnd(h) = target {
            let hwnd = to_hwnd(*h);
            return Ok(vec![Self::hwnd_to_window_info(hwnd)]);
        }

        let all = self.list_windows()?;
        let matched: Vec<WindowInfo> = all
            .into_iter()
            .filter(|w| Self::matches_target(w, target))
            .collect();

        Ok(matched)
    }

    fn find_window(&self, target: &WindowTarget) -> Result<WindowInfo> {
        let mut matched = self.find_windows(target)?;
        match matched.len() {
            0 => Err(DesktopError::WindowNotFound(format!("{:?}", target)).into()),
            1 => Ok(matched.remove(0)),
            _ => {
                let titles: Vec<String> = matched
                    .iter()
                    .map(|w| format!("\"{}\" (HWND: {})", w.title, w.hwnd))
                    .collect();
                Err(DesktopError::AmbiguousWindow(format!(
                    "{}件一致しました。HWNDで直接指定してください:\n{}",
                    titles.len(),
                    titles.join("\n")
                ))
                .into())
            }
        }
    }

    fn focus_window(&self, hwnd: usize) -> Result<()> {
        let hwnd = to_hwnd(hwnd);

        // 最小化されている場合はリストアしてからフォアグラウンドに移動
        let is_minimized = unsafe { IsIconic(hwnd) }.as_bool();
        if is_minimized {
            unsafe {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }

        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    fn set_window_state(&self, hwnd: usize, state: WindowState) -> Result<()> {
        let hwnd = to_hwnd(hwnd);
        let cmd = match state {
            WindowState::Minimize => SW_MINIMIZE,
            WindowState::Maximize => SW_MAXIMIZE,
            WindowState::Restore => SW_RESTORE,
        };
        unsafe {
            let _ = ShowWindow(hwnd, cmd);
        };
        Ok(())
    }

    fn resize_window(&self, hwnd: usize, width: u32, height: u32) -> Result<()> {
        let hwnd = to_hwnd(hwnd);
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                width as i32,
                height as i32,
                SWP_NOMOVE | SWP_NOZORDER,
            )?;
        }
        Ok(())
    }

    fn get_foreground_window(&self) -> Result<Option<WindowInfo>> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return Ok(None);
        }
        Ok(Some(Self::hwnd_to_window_info(hwnd)))
    }

    fn close_window(&self, hwnd: usize) -> Result<()> {
        let hwnd = to_hwnd(hwnd);
        unsafe {
            PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0))?;
        }
        Ok(())
    }
}
