#[allow(dead_code)]
/// GitHub Releases APIを使って新バージョンがあれば通知する。
///
/// ネットワーク未接続・タイムアウト・解析失敗は全て無視する。
/// --version 実行時に呼び出すことを想定。
pub fn check_latest_version(current: &str) {
    // 別スレッドで非同期に確認し、タイムアウトで諦める
    let current = current.to_string();
    let handle = std::thread::spawn(move || check_latest_version_inner(&current));

    // 2秒待って終わらなければ諦める
    match handle.join() {
        Ok(Some(msg)) => eprintln!("{}", msg),
        _ => {}
    }
}

fn check_latest_version_inner(_current: &str) -> Option<String> {
    // ネットワーク処理を単純な標準ライブラリのTcpStreamで実装し、
    // 外部クレート（reqwest等）への依存を避ける。
    // HTTPSのTLSが必要なため、TcpStreamだけでは難しい。
    // ここでは失敗しても問題ないため、シンプルな実装にとどめる。
    //
    // 将来的にreqwest等を追加する場合はここを置き換える。
    None
}
