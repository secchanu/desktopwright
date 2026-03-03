pub mod capture;
pub mod input;
pub mod ui_tree;
pub mod window;

use windows::Win32::Foundation::HWND;

use crate::core::platform::Platform;
use capture::WindowsScreenCapture;
use input::WindowsInputController;
use ui_tree::WindowsUiAutomation;
use window::WindowsWindowManager;

pub(crate) fn to_hwnd(hwnd: usize) -> HWND {
    HWND(std::ptr::with_exposed_provenance_mut(hwnd))
}

pub fn create_platform() -> Platform {
    Platform {
        window_manager: Box::new(WindowsWindowManager::new()),
        screen_capture: Box::new(WindowsScreenCapture::new()),
        input_controller: Box::new(WindowsInputController::new()),
        ui_automation: Box::new(WindowsUiAutomation::new()),
    }
}
