use anyhow::Result;
use clap::Args;

use crate::core::platform::InputController;

#[derive(Args, Debug)]
pub struct KeyArgs {
    /// 送信するキー（例: "ctrl+c", "enter", "f5", "shift+tab"）
    pub key: String,

    /// キー入力前の待機時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,
}

#[derive(Args, Debug)]
pub struct KeydownArgs {
    /// 押し続けるキー（単一キーのみ: "ctrl", "shift", "alt" など）
    pub key: String,
}

#[derive(Args, Debug)]
pub struct KeyupArgs {
    /// 離すキー（keydown で押したキーに対応）
    pub key: String,
}

pub fn run_key(args: &KeyArgs, input: &dyn InputController) -> Result<()> {
    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    input.send_key(&args.key)?;
    Ok(())
}

pub fn run_keydown(args: &KeydownArgs, input: &dyn InputController) -> Result<()> {
    input.send_key_down(&args.key)?;
    Ok(())
}

pub fn run_keyup(args: &KeyupArgs, input: &dyn InputController) -> Result<()> {
    input.send_key_up(&args.key)?;
    Ok(())
}
