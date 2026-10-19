//! Сохранения (§11.4, решение 15.10): версионированные файлы жизни и метапрогрессии и каталог, где
//! они лежат. Ядро собирает и читает текст файлов, каталог даёт игра — в Godot это `user://`.

mod save_error;
mod save_format;
mod storage;

pub use save_error::SaveError;
pub use save_format::{LIFE, META, SaveFormat};
pub use storage::Storage;
