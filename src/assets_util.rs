use std::path::PathBuf;

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn assets_dir() -> PathBuf {
    manifest_dir().join("assets")
}

pub fn card_defs_path() -> PathBuf {
    assets_dir().join("card_defs.ron")
}

/// Copy Windows 微软雅黑 into assets/fonts so Bevy can load CJK glyphs.
pub fn ensure_ui_font() {
    let dest = assets_dir().join("fonts").join("msyh.ttc");
    if dest.exists() {
        return;
    }
    #[cfg(windows)]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            let src = PathBuf::from(windir).join("Fonts").join("msyh.ttc");
            if src.exists() {
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::copy(&src, &dest);
            }
        }
    }
    let _ = dest;
}

pub fn ui_font_asset_path() -> &'static str {
    "fonts/msyh.ttc"
}

pub fn font_available() -> bool {
    assets_dir().join(ui_font_asset_path()).exists()
}
