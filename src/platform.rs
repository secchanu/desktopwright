#[cfg(target_os = "windows")]
pub mod windows;

use crate::core::platform::Platform;

pub fn create_platform() -> Platform {
    #[cfg(target_os = "windows")]
    return windows::create_platform();

    #[cfg(not(target_os = "windows"))]
    compile_error!("desktopwright は現在 Windows のみをサポートしています");
}
