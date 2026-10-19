<!-- MNEME-CRITERIA-CONTRACT -->
## mneme: phase criteria contract

- A done-when criterion is ONE argv command — no quotes, no `&&`/`|`: the gate-runner spawns a
  single process, and any quote or shell operator is an instant red `malformed-command` gate.
- Executable-first: a criterion is a command with exit 0 and a DEFINITE target (a specific test
  file, a `grep -q MARKER path`) — never a bare full-suite run alone. An agent-judged criterion
  is allowed only where the outcome is fundamentally not machine-checkable, and is marked
  `agent-judged` explicitly.
- Every criterion command must EXIST in the project (a script in package.json, a file on disk) —
  verify before writing the spec, never guess.
<!-- /MNEME-CRITERIA-CONTRACT -->

## Проект «Спекулянтъ»

- Godot строго 4.x: точная версия пинится в `tools.toml` (линейка 4.7), мост — крейт `godot` (godot-rust) с фичей `api-4-7`. Любой API Godot 3 — дефект; при сомнении сверяться с документацией Godot 4.
- Интерфейс пишется кодом: единственная сцена — `godot/main.tscn` с одной нодой; новые `.tscn` руками не создаются.
- Вся игровая логика живёт в `crates/speculant-core`; GDScript только показывает `StateView` и передаёт команды. `cargo xtask deps-check` сторожит ядро от Godot.
- Мост — класс `SpeculantGame` с четырьмя методами: `new_life(json)`, `advance(delta_ms)`, `apply(json)`, `view_json()`. Снимок `StateView` и команды `Command` — JSON; ответ на команду — `{"ok":true}` или `{"error": …, "message": …}`. Старт сеанса — `{"storage": <каталог игрока>, "life": {…}}`, `{"storage": …, "resume": <слот>}` или `{"storage": …}` — лобби: каталог открыт, жизнь не начата, `view_json()` отдаёт `LobbyView` (биографии, записная книжка, рекорды), а команды ждут жизни. Сохранения `saves/<слот>.spk` и `meta.spk` пишет ядро в каталог, который `godot/src/saves/saves.gd` берёт из `user://`.
- Интерфейс — стек экранов `godot/src/screens/router.gd`: сеанс открывается заставкой в лобби, жизнь стоит на экране режима, который называет ядро (рынок, зона, итоги), событие ложится поверх модальным окном; экраны отвечают на ввод намерениями (команда ядру, экран поверх, назад, новая жизнь, лобби, выход). Время идёт только на рынке, в пути и в зоне.
- Единый раннер гейтов — `cargo xtask <команда>`; Godot скачивается командой `cargo xtask godot-fetch` в `.tools/` и работает в self-contained режиме (данные редактора и шаблоны экспорта — в `.tools/godot/<версия>/editor_data/`).
- Релиз: версия одна — `[workspace.package] version` в `Cargo.toml`, `godot/project.godot` повторяет её (`cargo xtask version-check`); экспорт платформы — `cargo xtask export <платформа>` на её родной системе, гейт — `cargo xtask export-check`; по тегу `v<версия>` `.github/workflows/release.yml` собирает Windows x86_64, Linux x86_64 и macOS universal и прикладывает архивы к релизу.
- Строки игры — русские и только из набора глифов шрифта (`godot/assets/fonts/glyphs.txt`); проверка — `cargo xtask charset-check`.
- Экран — один узел `TextScreen` 80×25; макеты дизайн-книги сняты в `design/etalons/`, сверка экрана — `cargo xtask screen-check <экран>`.
- Контент — TOML в `data/`, файл на сущность, имя файла — идентификатор (решение 15.7; формат — `docs/CONTENT.md`). Проверка — `cargo run -p speculant-validate -- data [--strict] [--watch]`, покрытие событий прогоном ботов — `cargo xtask coverage-check`, заготовка события — `cargo xtask new-event <эпоха> <имя>`. Мост вшивает `data/` при сборке.
- Числовые пороги объёма, баланса и покрытия — только в `data/balance.toml`, никогда в коде и критериях.
- Цены и суммы в данных — реальные рубли 1984 года (`Real`); деньги игрока — `Money` в своей валюте (рубли — в копейках до деноминации 1998 года), и переводятся они друг в друга и в строки только через `MoneyRules` (`data/economy/money.toml`).
