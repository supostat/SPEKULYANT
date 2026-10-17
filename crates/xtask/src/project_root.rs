use std::path::{Path, PathBuf};

/// Корень репозитория: xtask живёт в `crates/xtask`, два уровня вверх.
pub fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/xtask лежит на два уровня ниже корня")
        .to_path_buf()
}
