use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use super::SaveError;

const FORMAT_KEY: &str = "format";
const VERSION_KEY: &str = "version";
const CONTENT_KEY: &str = "content";
const BODY_KEY: &str = "body";
const FIRST_VERSION: u32 = 1;
const PAGER_KEY: &str = "pager";
const PRICE_HISTORY_KEY: &str = "price_history";
const SEEN_PRICES_KEY: &str = "seen_prices";

/// Вид файла и версия формата его тела. Когда тело меняется, версия растёт и добавляется шаг
/// миграции: `migrations[i]` переводит тело версии `i + 1` в версию `i + 2`, поэтому файлы любой
/// прошлой версии читаются по цепочке шагов.
pub struct SaveFormat {
    pub kind: &'static str,
    pub version: u32,
    pub migrations: &'static [fn(Value) -> Value],
}

/// Файл жизни: мир одной жизни. Версия 2 добавила ленту пейджера, версия 3 — историю цен,
/// версия 4 — цены, какими игрок видел толкучки.
pub const LIFE: SaveFormat = SaveFormat {
    kind: "speculant-life",
    version: 4,
    migrations: &[add_pager, add_price_history, add_seen_prices],
};

/// Файл метапрогрессии: открытия, книжка, история и рекорды.
pub const META: SaveFormat = SaveFormat {
    kind: "speculant-meta",
    version: 1,
    migrations: &[],
};

/// Жизнь версии 1 — до пейджера: лента пуста, пришедшие раньше сообщения не сохранялись.
fn add_pager(mut body: Value) -> Value {
    if let Value::Object(world) = &mut body {
        world.insert(PAGER_KEY.to_owned(), json!({ "messages": [] }));
    }
    body
}

/// Жизнь версии 2 — до истории цен: история пуста и начнётся с ближайшей недели.
fn add_price_history(mut body: Value) -> Value {
    if let Value::Object(world) = &mut body {
        world.insert(
            PRICE_HISTORY_KEY.to_owned(),
            json!({ "newest": null, "weeks": {} }),
        );
    }
    body
}

/// Жизнь версии 3 — до памяти о ценах: игрок ещё ниоткуда не уезжал с ней.
fn add_seen_prices(mut body: Value) -> Value {
    if let Value::Object(world) = &mut body {
        world.insert(SEEN_PRICES_KEY.to_owned(), json!({ "cities": {} }));
    }
    body
}

impl SaveFormat {
    /// Текст файла: вид, версия, отпечаток данных игры, если файл к ним привязан, и тело.
    pub fn encode<T: Serialize>(&self, content: Option<u64>, body: &T) -> String {
        let mut file = Map::new();
        file.insert(FORMAT_KEY.to_owned(), Value::from(self.kind));
        file.insert(VERSION_KEY.to_owned(), Value::from(self.version));
        if let Some(content) = content {
            file.insert(CONTENT_KEY.to_owned(), Value::from(content));
        }
        file.insert(
            BODY_KEY.to_owned(),
            serde_json::to_value(body).expect("состояние ядра сериализуется в JSON"),
        );
        Value::Object(file).to_string()
    }

    /// Тело файла: вид должен совпасть, версия — не новее этой, отпечаток данных — тот же, если
    /// он задан; тело прошлой версии проходит шаги миграции.
    pub fn decode<T: DeserializeOwned>(
        &self,
        text: &str,
        content: Option<u64>,
    ) -> Result<T, SaveError> {
        let mut file: Map<String, Value> =
            serde_json::from_str(text).map_err(|error| SaveError::Corrupt(error.to_string()))?;
        if file.get(FORMAT_KEY).and_then(Value::as_str) != Some(self.kind) {
            return Err(SaveError::OtherFormat);
        }
        let version = file
            .get(VERSION_KEY)
            .and_then(Value::as_u64)
            .and_then(|version| u32::try_from(version).ok())
            .filter(|version| *version >= FIRST_VERSION)
            .ok_or_else(|| SaveError::Corrupt("нет версии формата".to_owned()))?;
        if version > self.version {
            return Err(SaveError::Newer {
                version,
                supported: self.version,
            });
        }
        if let Some(content) = content
            && file.get(CONTENT_KEY).and_then(Value::as_u64) != Some(content)
        {
            return Err(SaveError::OtherContent);
        }
        let body = file
            .remove(BODY_KEY)
            .ok_or_else(|| SaveError::Corrupt("нет тела".to_owned()))?;
        let steps = usize::try_from(version - FIRST_VERSION).expect("версий мало");
        let body = self.migrations[steps..]
            .iter()
            .fold(body, |body, migrate| migrate(body));
        serde_json::from_value(body).map_err(|error| SaveError::Corrupt(error.to_string()))
    }
}
