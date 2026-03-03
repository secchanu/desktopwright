use thiserror::Error;

#[derive(Error, Debug)]
pub enum DesktopError {
    #[error("ウィンドウが見つかりません: {0}")]
    WindowNotFound(String),

    #[error("ウィンドウを特定できません（複数のウィンドウが一致します）: {0}")]
    AmbiguousWindow(String),

    #[error("キャプチャがブロックされています（DRMまたはセキュリティ保護）")]
    CaptureBlocked,
}
