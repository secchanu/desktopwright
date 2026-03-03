use anyhow::Result;
use clap::Args;

use crate::core::platform::InputController;

#[derive(Args, Debug)]
pub struct TypeArgs {
    /// 入力するテキスト
    pub text: String,

    /// 入力前の待機時間（ミリ秒）
    #[arg(long)]
    pub delay: Option<u64>,
}

pub fn run_type(args: &TypeArgs, input: &dyn InputController) -> Result<()> {
    if let Some(ms) = args.delay {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    input.type_text(&args.text)?;
    Ok(())
}
