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
- Единый раннер гейтов — `cargo xtask <команда>`; Godot скачивается командой `cargo xtask godot-fetch` в `.tools/`.
- Строки игры — русские и только из набора глифов шрифта (`godot/assets/fonts/glyphs.txt`, появляется в фазе text-screen).
- Числовые пороги объёма, баланса и покрытия — только в `data/balance.toml`, никогда в коде и критериях.
