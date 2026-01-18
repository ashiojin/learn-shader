// build.rs
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    // Windows MSVC ターゲットのみ対象
    if target.contains("msvc") {
        // 1. 環境変数のチェック
        let xwin_cache_env = "XWIN_CACHE_DIR";
        let xwin_path = match std::env::var(xwin_cache_env) {
            Ok(path) => path,
            Err(_) => {
                // 環境変数が設定されていない場合、エラーメッセージを表示して終了
                panic!(
                    "\n\n[Error] xwin のパスが設定されていません。\n\
                    以下の手順でセットアップを行ってください：\n\n\
                    1. xwin をインストール (未実行の場合):\n\
                       cargo install xwin\n\n\
                    2. MSVC SDK をダウンロード・展開:\n\
                       xwin --accept-license splat --output /path/to/xwin-data\n\n\
                    3. 環境変数をセット:\n\
                       export {}=/path/to/xwin-data\n\n\
                    注意: /path/to/xwin-data は絶対パスを指定してください。\n",
                    xwin_cache_env
                );
            }
        };

        // 2. ターゲットアーキテクチャの判定
        let arch = if target.contains("x86_64") {
            "x86_64"
        } else if target.contains("aarch64") {
            "aarch64"
        } else {
            "x86"
        };

        // 3. リンカパスの指定
        // xwin splat 後のディレクトリ構造 (sdk/lib/um, sdk/lib/ucrt, crt/lib)
        let lib_dirs = [
            format!("{}/sdk/lib/um/{}", xwin_path, arch),
            format!("{}/sdk/lib/ucrt/{}", xwin_path, arch),
            format!("{}/crt/lib/{}", xwin_path, arch),
        ];

        for dir in &lib_dirs {
            // ディレクトリの存在チェック（オプション）
            if !std::path::Path::new(dir).exists() {
                println!("cargo:warning=Directory not found: {}", dir);
            }
            println!("cargo:rustc-link-search=native={}", dir);
        }

        // 変更を検知するための再ビルドトリガー
        println!("cargo:rerun-if-env-changed={}", xwin_cache_env);
    }
}
