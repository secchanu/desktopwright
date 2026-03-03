use anyhow::{anyhow, Result};
use clap::Args;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;

static SKILL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills/desktopwright");

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// AIエージェント統合スキルをインストールする
    #[arg(long)]
    pub skills: bool,

    /// グローバルインストール（~/.claude/skills/desktopwright/）
    /// 指定しない場合はカレントディレクトリに .claude/skills/desktopwright/ を作成する
    #[arg(long)]
    pub global: bool,
}

pub fn run_install(args: &InstallArgs) -> Result<()> {
    if !args.skills {
        return Err(anyhow!("--skills フラグを指定してください: desktopwright install --skills"));
    }

    let target = resolve_target(args.global)?;
    std::fs::create_dir_all(&target)?;

    SKILL_DIR
        .extract(&target)
        .map_err(|e| anyhow!("スキルの展開に失敗しました: {}", e))?;

    eprintln!("スキルをインストールしました: {}", target.display());
    Ok(())
}

fn resolve_target(global: bool) -> Result<PathBuf> {
    if global {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_err(|_| anyhow!("ホームディレクトリが見つかりません（USERPROFILE / HOME が未設定）"))?;
        Ok(PathBuf::from(home).join(".claude/skills/desktopwright"))
    } else {
        Ok(std::env::current_dir()?.join(".claude/skills/desktopwright"))
    }
}
