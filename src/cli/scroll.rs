use anyhow::Result;
use clap::{Args, ValueEnum};

use crate::core::platform::InputController;
use crate::core::types::ScrollDirection;

#[derive(Args, Debug)]
pub struct ScrollArgs {
    /// スクロール方向
    #[arg(long, short = 'd')]
    pub direction: DirectionArg,

    /// スクロール量（ホイールノッチ数、デフォルト 3）
    #[arg(long, short = 'n', default_value = "3")]
    pub amount: i32,

    /// スクロール位置のX座標（省略時は現在位置）
    #[arg(long, short = 'x')]
    pub x: Option<i32>,

    /// スクロール位置のY座標（省略時は現在位置）
    #[arg(long, short = 'y')]
    pub y: Option<i32>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum DirectionArg {
    Up,
    Down,
    Left,
    Right,
}

impl From<DirectionArg> for ScrollDirection {
    fn from(d: DirectionArg) -> Self {
        match d {
            DirectionArg::Up => ScrollDirection::Up,
            DirectionArg::Down => ScrollDirection::Down,
            DirectionArg::Left => ScrollDirection::Left,
            DirectionArg::Right => ScrollDirection::Right,
        }
    }
}

pub fn run_scroll(args: &ScrollArgs, input: &dyn InputController) -> Result<()> {
    input.mouse_scroll(args.direction.into(), args.amount, args.x, args.y)?;
    Ok(())
}
