use anyhow::Result;
use serde::Serialize;

/// 出力フォーマットの選択肢
#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
}

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", json);
    Ok(())
}
