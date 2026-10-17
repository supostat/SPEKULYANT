<!--
  SPEC-SPEKULYANT.md — спека реализации игры «Спекулянтъ» целиком.
  Прогон: миграция финалом /mneme:plan → /mneme:dev spekulyant.

  ИСТОЧНИКИ ИСТИНЫ: docs/SPEKULYANT.md (§1–§15) — главный; design/design-book.html
  (§I–§XV, шесть макетов: заставка, рынок, событие, управляющий, пейджер, итоги жизни) —
  эталон интерфейса. При расхождении прав docs/SPEKULYANT.md.

  Выбран вариант C веера: каркас стека первым (узел 80×25 + эталоны, контракт моста),
  затем ядро снизу вверх, контент и симулятор, экраны пакетом по стабильному контракту.
  Отвергнуты A (ядро снизу вверх, интерфейс в конце — риски DOS-рендера и gdext
  обнаруживаются поздно) и B (вертикальные ломтики — мост и GDScript правятся в каждой
  фазе при меняющемся API gdext).

  Ревью, круг 1 (цифра 2): пороги объёма контента вынесены в решение 15.11 и
  data/balance.toml; балансовые пороги калибруются в фазе simulator по измеренной базе;
  извлечение эталонов переведено с Node на движок Boa внутри xtask.

  Жёсткие решения, которые фазы не пересматривают: Godot 4; ядро — чистый Rust-крейт без
  Godot; мост gdext минимальный; интерфейс GDScript; вся логика в ядре; экран 80×25 одним
  узлом с буфером символов и цветов EGA; интерфейс кодом; цены в реальных единицах,
  номинал через индекс инфляции; детерминизм по сиду; контент — данные; CLI-симулятор и
  CLI-валидатор; Windows / macOS universal / Linux; только русский. Без MVP и версий.
-->

# Baseline

- **Prior spec:** нет — первая спека проекта. Исходники истины: `docs/SPEKULYANT.md`
  §1–§15 и `design/design-book.html` §I–§XV с шестью макетами.
- **Prior spec-hash:** sha256:<placeholder — подтвердить>.
- **Что уже есть и НЕ трогается:** `docs/SPEKULYANT.md` (меняется только решениями §15,
  включая добавленный планом вопрос 15.11, и их следствиями в соответствующих разделах),
  `design/design-book.html` (меняется только маркерами решений в §II/§XV и заменой глифа
  ▶ → ► в макете рынка), `CLAUDE.md` с контрактом критериев (дополняется проектной
  секцией, блок mneme не трогается), `.gitignore` (дополняется).
- **Автоматика репо:** не найдена — ни CI, ни манифестов, ни релизных скриптов, ни версии.
  Фаза `release` создаёт `.github/workflows/release.yml` с проверкой согласованности версии
  (`cargo xtask version-check`). Отдельной bump-фазы НЕТ: до первого релиза поднимать
  нечего, а после него подъём версии — действие релиза, не фаза плана.
- **Тулчейн (разведано, не предположено):** cargo/rustc 1.94 с таргетами
  aarch64-apple-darwin, x86_64-apple-darwin, x86_64/aarch64-unknown-linux-musl; Godot на
  машине ОТСУТСТВУЕТ — его скачивает пином `cargo xtask godot-fetch` в `.tools/`. Node не
  требуется ни игре, ни разработчику: JS дизайн-книги исполняет xtask. `just` есть, но не
  используется: один раннер гейтов — `cargo`.
- **Границы правил-генераторов.** TYPECHECK-CRITERION-RULE: typecheck-скрипта сегодня нет;
  фаза `bootstrap` создаёт его аналоги — `cargo clippy --workspace --all-targets -- -D
  warnings` для Rust и `cargo xtask gd-check` для GDScript — и они идут критерием в каждой
  кодовой фазе соответствующего языка. WIRE-PHASE-RULE: применяется — терминальная фаза
  `wire`. REAL-DEP-SMOKE-RULE: движок строит deps цепочкой по порядку записи; первая задача
  каждой фазы называет, что она потребляет из предыдущих, а smoke-критерий — её тестовый
  файл, который компилируется против настоящих модулей (моков внутри крейта нет по
  построению); кросс-крейтовые рёбра (sim, validate, godot → core) гоняют настоящий core;
  чисто порядковые рёбра помечены `order-only`. VERSION-BUMP-RULE: см. «Автоматика».
  ETALON-ACCEPTANCE-RULE: эталон HTML — чеклист приёмки в Knowledge.
- Обоснование выбора — staged decision note (id после accept).

# Stack

- `Cargo.toml` — workspace (resolver 2), `rust-toolchain.toml`, `.cargo/config.toml` с
  алиасом `xtask`, `tools.toml` — пин версии Godot и sha256 по платформам.
- `crates/speculant-core/` — библиотека ядра без Godot: `time/`, `rng/`, `content/`,
  `economy/`, `reputation/`, `events/`, `travel/`, `npc/`, `business/`, `life/`, `zone/`,
  `history/`, `meta/`, `save/`, `view/` (StateView, Command, фасад `Game`).
- `crates/speculant-sim/` — CLI-симулятор баланса: боты-стратегии, тысячи жизней, метрики.
- `crates/speculant-validate/` — CLI-валидатор контента: схема, ссылки, достижимость.
- `crates/speculant-godot/` — cdylib gdext: класс `SpeculantGame` с четырьмя методами.
- `crates/xtask/` — сквозные команды проекта: `godot-fetch`, `godot-check`, `gd-check`,
  `font-glyphs`, `charset-check`, `etalons` (движок Boa + `src/etalons/dom_shim.js`),
  `screen-check`, `screen-smoke`, `screenshot`, `bridge-smoke`, `ui-flow-smoke`,
  `sound-smoke`, `content-count`, `balance-check`, `coverage-check`, `deps-check`,
  `docs-check`, `version-check`, `build-godot`, `export-check`, `e2e`, `new-event`.
- `godot/` — проект Godot 4: `project.godot`, `speculant.gdextension`, `main.tscn` (одна
  корневая нода со скриптом), `src/text_screen/`, `src/screens/`, `src/input/`,
  `src/sound/`, `src/crt/`, `src/saves/`, `tools/` (безголовые скрипты для xtask),
  `assets/fonts/` (шрифт пака с лицензией, список глифов).
- `data/` — контент: `goods/`, `cities/`, `routes.toml`, `epochs/`, `biographies/`, `npc/`,
  `events/<epoch>/`, `history/`, `zone/`, `sound/`, `balance.toml` (секции `[volume]` из
  решений 15.2 и 15.11, `[balance]` из калибровки, `[coverage]`).
- `design/etalons/` — извлечённые эталоны экранов (`<screen>.screen`, `<screen>.view.json`,
  `palette.txt`).
- `docs/CONTENT.md` — руководство авторинга контента; `docs/SPEKULYANT.md` — решения §15.
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`.

# Conventions

- Godot строго 4.x: версия пинится в `tools.toml` и в проектной секции `CLAUDE.md`; любой
  API Godot 3 — дефект. Версия gdext подбирается по официальной таблице совместимости
  godot-rust на момент фазы `bootstrap`, не из памяти.
- `speculant-core` не зависит от Godot ни прямо, ни транзитивно — `cargo xtask deps-check`
  это сторожит.
- Вся игровая логика в ядре. `StateView` несёт готовые к показу строки (деньги в номинале,
  даты, курсы); GDScript никогда не считает игровые величины и не хранит игровое состояние.
- Мост — четыре метода: `new_life(json)`, `advance(delta_ms) -> bool`, `apply(json) -> json`,
  `view_json() -> String`. Сериализация одна — serde_json; сохранения — serde с версией формата.
- Детерминизм: единый `WorldRng` (ChaCha8) с именованными подпотоками по подсистемам;
  трансцендентные функции в ядре только из крейта `libm` (одинаковый результат на трёх
  платформах); `tests/determinism.rs` растёт с каждой подсистемой.
- Цены и капитал в ядре — `Real` (i64 в копейках 1984 года); номинал — функция от `Real`,
  даты и валюты через индекс инфляции.
- Интерфейс кодом: экраны — классы GDScript с `draw(view, screen)` и
  `handle(event) -> Command или null`; единственная сцена — `main.tscn` с одной нодой.
- Экран сверяется по буферу: формат `.screen` — 25 строк по 80 символов, пустая строка, 25
  строк hex-цветов переднего плана, пустая строка, 25 строк hex-цветов фона. Эталон
  извлекается из дизайн-книги её собственным JS, исполняемым в xtask движком Boa;
  PNG-скриншот — для приёмки глазами.
- Все строки игры — русские и из набора глифов шрифта (`godot/assets/fonts/glyphs.txt`,
  генерируется xtask из TTF); `cargo xtask charset-check` сторожит `data/` и `godot/src/`.
- Контент — TOML, один файл на сущность, имя файла равно `id` (snake_case ASCII); условия
  событий — структурные таблицы, не строки-выражения; подстановки в тексте — `{npc}`,
  `{price}`, `{sum}`.
- Числовые пороги объёма, баланса и покрытия живут только в `data/balance.toml`; критерии
  спеки их не дублируют — команды `content-count`, `balance-check`, `coverage-check` читают
  файл.
- Решения §15 принимаются с пользователем в первой задаче фазы и фиксируются в
  `docs/SPEKULYANT.md` видимой строкой `**Решено** (DECIDED-15-N): …` в §15 плюс правкой
  раздела, которого касаются; если решение меняет содержимое дизайн-книги — тот же маркер в
  её §XV. `cargo xtask docs-check` сверяет маркеры в обоих документах; `--all-decided`
  требует маркер у каждого нумерованного пункта §15.
- Тесты ядра — интеграционные файлы `crates/speculant-core/tests/<subsystem>.rs`; фикстуры
  в `tests/fixtures/`; тесты — спецификации, следуют за кодом.
- Все done-when — один argv без shell: `cargo …`, `cargo xtask …`, `grep -qF МАРКЕР путь`.

# Knowledge

- Параметры принятых решений живут в `docs/SPEKULYANT.md`, не в этой спеке: спека только
  называет вопрос, варианты и рекомендацию. Ниже одиннадцать решений §15 (десять из
  документа и одно, добавленное планом), каждое привязано к фазе; принимать ДО остальных
  задач фазы, вариант выбирается цифрой.
- Решение 15.3 (фаза `core-time`) — масштаб времени. Варианты: 1) как в макете рынка
  дизайн-книги: день за 1,8 / 1,0 / 0,5 с на скоростях 1–3, пауза на событиях, перемотка «до
  события» и «до прибытия»; 2) 3 / 1,5 / 0,5 с; 3) 1,0 / 0,5 / 0,25 с и перемотка только до
  события. Рекомендация: 1 — макет уже так живёт, а 6–10 часов на жизнь из §11 набираются
  с паузами на событиях и временем в экранах.
- Решение 15.7 (фаза `content-format`) — формат данных и авторинг. Варианты: 1) TOML по
  файлу на сущность, схема в serde, валидатор с режимом `--watch`, авторинг в любом
  редакторе, `cargo xtask new-event` для заготовок, `docs/CONTENT.md`; 2) JSON и
  редактор-плагин внутри Godot; 3) RON. Рекомендация: 1 — текст с комментариями,
  diff-friendly, без кода на GDScript; редактор в Godot — отдельный продукт, который
  задержит контент, главный риск проекта (§10.3).
- Решение 15.6 (фаза `core-reputation`) — фракции. Варианты: 1) шесть фракций §9: бандиты,
  милиция, чиновники, торговцы, народ, силовики, причём силовики появляются в 1999-м; 2) одна
  линия «государство», переименовываемая по эпохам (ОБХСС → налоговая полиция → силовики);
  3) пять фракций без силовиков. Рекомендация: 1 — появление новой фракции само
  рассказывает эпоху.
- Решение 15.4 (фаза `core-life`) — быт персонажа. Варианты: 1) здоровье — скрытая шкала с
  видимыми состояниями, жильё — ресурс со статусом и местом «под матрасом», семья — только
  события; 2) всё только события; 3) здоровье, жильё и семья как полные сущности с расходами.
  Рекомендация: 1 — финал «смерть от здоровья и пьянства» (§11.2) требует шкалы, жильё —
  мотиватор (§11), семья событиями сохраняет сатиру без микроменеджмента.
- Решение 15.5 (фаза `core-life`) — биографии. Варианты: 1) пять стартовых из §11.1 плюс две
  открываемые с условиями по финалам (номенклатура, зона); 2) три стартовых (студент,
  инженер, таксист) и четыре открываемых; 3) все семь доступны сразу. Рекомендация: 1 —
  открываемость и есть метапрогрессия (§11.3). Макет «Итоги жизни» упоминает «Бывшего
  кооператора» — документ главнее, биография добавляется только решением.
- Решение 15.8 (фаза `core-zone`) — механика зоны. Варианты: 1) срок в реальном календаре,
  тик зоны — неделя, свои события по объёму из 15.11, экономика сигарет и чая, доверенные
  лица — управляющие с политикой «на время отсидки», выход по сроку / УДО / амнистии / за
  взятку; 2) зона как цепочка событий без экономики; 3) зона как второй полный рынок.
  Рекомендация: 1 — ровно «отдельный режим, а не пропуск времени» (§12) без второго рынка.
- Решение 15.9 (фаза `core-history`) — генератор после 2004-го. Варианты: 1) библиотека
  параметрических шаблонов (валютный шок, банковский кризис, налоговая реформа,
  запрет/разрешение товара, технологическая волна) с интервалами и весами по сиду; 2) повтор
  исторических событий в случайном порядке; 3) только ценовой шум. Рекомендация: 1 — «по тем
  же механизмам» (§3.5) и не ломает четвёртую стену повтором МММ.
- Решение 15.10 (фаза `core-meta`) — рекорды. Варианты: 1) только локальная таблица в
  `user://`, раздельно ironman и свободный режим; 2) локальная плюс экспорт в текстовый файл;
  3) общая онлайн-таблица. Рекомендация: 1 — нет сервера и аккаунтов, игра одиночная.
- Решение 15.1 (фаза `content-world`) — города и множители. Варианты: 1) десять городов:
  Москва, Ленинград, провинциальный центр, Рига, Ташкент, Тбилиси, Варшава, Стамбул, Пекин,
  Берлин («по блату» до 1990) с множителями по категориям товаров; 2) 16–18 городов с
  региональными кластерами; 3) восемь типов-слотов с именами по сиду. Рекомендация: 1 —
  «десяток городов» (§1), меньше городов — больше событий на город.
- Решение 15.2 (фаза `content-world`) — ассортимент. Варианты: 1) 40 товаров в 8 категориях
  с опорными точками кривых раз в два года; 2) 60 товаров; 3) 30 товаров. Рекомендация: 1 —
  середина диапазона §5.3, каждая категория рассказывает эпоху, 60 размывают авторский труд.
  Принятое число пишется в `data/balance.toml` `[volume] goods`.
- Решение 15.11 (фаза `content-world`, вопрос добавляется планом в §15 документа) — объём
  контента: события по эпохам, события зоны, шаблоны песочницы, пул шаблонов NPC. Варианты:
  1) около 330 событий — застой 70, перестройка 70, дикий рынок 90, стабилизация 60, зона
  40; шаблонов после 2004-го 20; пул NPC 20; 2) компактный, около 220 — 50 / 50 / 60 / 40,
  зона 30, шаблонов 15, пул NPC 16; 3) широкий, около 480 — 100 / 100 / 150 / 80, зона 50,
  шаблонов 30, пул NPC 25. Рекомендация: 1 — «сотни событий» (§10) при главном риске проекта
  в объёме контента (§10.3); дикий рынок самый насыщенный по §3.3. Принятые числа пишутся в
  `data/balance.toml` `[volume]`, и только оттуда их читает `content-count`.
- Балансовые пороги (смертность по годам, дни без событий по эпохам, капитал стратегий
  выше N медиан) планом НЕ назначаются: фаза `simulator` снимает базовые метрики первого
  прогона, предлагает пороги от измеренной базы, пользователь подтверждает цифрой, и они
  записываются в `data/balance.toml` `[balance]` со строкой происхождения
  `# calibrated: seed, lives, date`. То же для `[coverage] lives` валидатора.
- Приёмка экранов по эталону HTML (ETALON-ACCEPTANCE-RULE): для каждого из шести макетов —
  раскладка окон с координатами класса `Scr` дизайн-книги, полный набор состояний макета
  (рынок: идёт / пауза / скорости 1–3; событие: вопрос и три исхода; управляющий: все
  положения шести политик), дозировка цвета (красное окно только у события, жёлтый — выборы и
  предупреждения), единицы — символы 80×25 и цвета 0–15, никаких пикселей. Экраны без макета
  (поездка, связи, деньги, записная книжка, зона, выбор биографии) принимаются по правилам
  §XIII дизайн-книги и PNG-скриншоту; это приёмка человеком, не agent-judged критерий.
- Честный предел безголового Godot: он не рендерит пиксели, поэтому машинный гейт экрана —
  сравнение буфера узла с эталоном; PNG снимается `cargo xtask screenshot` в оконном режиме
  (в CI на Linux — под Xvfb) и смотрится человеком.
- Извлечение эталонов: xtask исполняет inline-скрипт дизайн-книги движком Boa (JavaScript
  на чистом Rust, крейт `boa_engine`) с JS-прелюдией `dom_shim.js` (document.getElementById
  со стором innerHTML, createElement, querySelectorAll, addEventListener, matchMedia,
  setInterval как no-op, без IntersectionObserver); собранный `innerHTML` каждого `scr-*`
  разбирается по span'ам классов `bN fM` в сетку `.screen`. Макеты — вывод собственного кода
  книги, поэтому порт на Rust разошёлся бы с ней. Риск: Boa не исполнит какую-то
  конструкцию книги; правило — упрощать JS книги (она наша), не добавлять Node.
- Шрифт пака VileR — Unicode TTF с кириллицей и псевдографикой; «CP866» в документе — набор
  глифов, а не кодировка файлов: игра хранит UTF-8. Макет рынка использует ▶ (U+25B6),
  которого в наборе нет — заменяется на ► (U+25BA) и в книге, и в игре.
- Коррекция 4:3: логический экран 720×400, узел растянут по вертикали ×1,35 до 720×540
  (720 : 540 = 4 : 3; множитель 1,2 верен только для 640×400),
  окно — `display/window/stretch/mode=viewport`, `aspect=keep`, `scale_mode=integer`.
- Плавающая история: `data/history/*.toml` задают окно дат, масштаб и альтернативные
  исходы; генератор при создании мира фиксирует `Timeline` по сиду, события ссылаются на
  узлы таймлайна условием `timeline = "..."`.
- Сохранения: жизнь — `user://saves/<slot>.spk` (ironman — один слот, автосохранение
  каждый день и после события, удаляется в финале; свободный — именованные слоты);
  метапрогрессия и рекорды — отдельный `user://meta.spk`, не зависит от режима.
- Боты симулятора: `random`, `arbitrage`, `hoarder`, `entrepreneur`, `cautious`, `reckless`;
  метрики: кривая выживания по годам, распределение капитала, гистограмма финалов, счётчики
  срабатываний событий (покрытие), дни без событий по эпохам, стратегии с капиталом выше
  N медиан (ломающие экономику).
- Валидатор: статически — схема, битые ссылки (товары, города, NPC, флаги, события цепочек),
  условия вне окна 1984–2004+, флаги, которые никто не ставит; динамически — покрытие по
  прогону симулятора на `[coverage] lives` жизней: событие без единого срабатывания —
  ошибка, если не помечено `rare = true`.
- macOS universal: cdylib собирается для aarch64 и x86_64 и склеивается `lipo` в
  `cargo xtask build-godot --universal`; Windows и Linux — на родных раннерах CI.

# Gameplan

Двадцать девять фаз, deps — цепочкой в порядке записи. Первая задача каждой фазы называет
основание в документах и что фаза потребляет из предыдущих.

### Phase 1: bootstrap — репозиторий, крейты, проект Godot, xtask, CI

- [ ] Основание: docs/SPEKULYANT.md §14.1, §14.4, §14.5; design/design-book.html §XIV
  (схема архитектуры). Зависимостей нет — первая фаза.
- [ ] Cargo-workspace: `Cargo.toml` (resolver 2, `workspace.package.version = 0.1.0`),
  `rust-toolchain.toml` (stable, канал пином), `.cargo/config.toml` с алиасом
  `xtask = run -p xtask --`; крейты `speculant-core` (lib: serde, serde_json, rand_chacha,
  libm), `speculant-sim` (bin), `speculant-validate` (bin), `speculant-godot` (cdylib:
  godot), `xtask` (bin) — каждый с пустым модулем и одним тестом.
- [ ] Версии: Godot 4 — последний стабильный на момент фазы, gdext — по таблице
  совместимости godot-rust; записать в `tools.toml` (версия, sha256 архивов для
  darwin/linux/windows) и в `Cargo.toml` крейта моста.
- [ ] `godot/project.godot` (Godot 4, рендер compatibility, окно 720×540, stretch viewport /
  keep / integer), `godot/speculant.gdextension` с путями библиотек по платформам,
  `godot/main.tscn` с одной нодой и `godot/src/main.gd`.
- [ ] `cargo xtask godot-fetch`: скачать пин Godot в `.tools/godot/<версия>/`, проверить
  sha256 до chmod; `cargo xtask godot-check`: `--headless --path godot --import --quit`.
- [ ] `cargo xtask gd-check`: для каждого `godot/src/**/*.gd` и `godot/tools/*.gd` —
  `--headless --check-only --script`, первая ошибка останавливает с путём файла.
- [ ] `cargo xtask deps-check`: по `cargo metadata` замыкание зависимостей
  `speculant-core` не содержит крейтов с префиксом `godot`.
- [ ] `cargo xtask docs-check`: маркеры `DECIDED-15-N` в `docs/SPEKULYANT.md` и
  `design/design-book.html` совпадают как множества; флаг `--all-decided` требует маркер у
  каждого нумерованного пункта §15 документа.
- [ ] `.gitignore` += `.tools/`, `godot/.godot/`, `godot/bin/`; `.github/workflows/ci.yml`:
  матрица ubuntu / macos / windows — fmt, clippy, test, godot-fetch, godot-check, gd-check,
  deps-check.
- [ ] `CLAUDE.md`: проектная секция под блоком mneme — версия Godot 4 и gdext, запрет API
  Godot 3 и ручных `.tscn`, «логика только в ядре», единый раннер гейтов `cargo xtask`,
  русские строки из набора глифов шрифта, пороги только в `data/balance.toml`.

**Done when (EXECUTABLE):**
```
cargo test --workspace
```
- Все пять крейтов компилируются и их стартовые тесты зелёные (exit 0).

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Typecheck-аналог проекта для Rust; с этой фазы идёт критерием в каждой Rust-фазе.

**Done when (EXECUTABLE):**
```
cargo fmt --all --check
```
- Форматирование единое с первого коммита кода.

**Done when (EXECUTABLE):**
```
cargo xtask godot-check
```
- Пин Godot скачан с проверкой sha256, проект импортируется безголово (exit 0).

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- Typecheck-аналог для GDScript; с этой фазы идёт критерием в каждой GDScript-фазе.

**Done when (EXECUTABLE):**
```
cargo xtask deps-check
```
- Ядро без Godot — сторож на месте и зелёный.

**Done when (EXECUTABLE):**
```
grep -qF godot-check .github/workflows/ci.yml
```
- CI гоняет гейт Godot, а не только cargo.

### Phase 2: text-screen — узел 80×25, примитивы, шрифт, палитра, эталоны, заставка

- [ ] Основание: docs/SPEKULYANT.md §13.1, §13.2 (заставка), §14.2; design/design-book.html
  §XIII, макет «Заставка» (`scr-boot`), класс `Scr` (put / fill / box / shadow / fkeys),
  классы цветов b0–b7 / f0–f15. Потребляет из `bootstrap`: проект Godot и xtask.
- [ ] `godot/src/text_screen/text_screen.gd` (`class_name TextScreen`): буфер 80×25 ячеек
  (символ, цвет 0–15, фон 0–7), методы `clear`, `put`, `fill`, `box` (одинарная и двойная
  рамка, заголовок), `shadow`, `fkeys`, `dump() -> String` в формате `.screen`; отрисовка в
  `_draw()` глифами шрифта `godot/assets/fonts/` (FontFile, сглаживание и хинтинг
  выключены, размер 16, шаг 9 px) на логическом поле 720×400 с масштабом ×1,35 по вертикали.
- [ ] `godot/src/text_screen/ega.gd`: шестнадцать цветов EGA константами; шрифт пака VileR с
  файлом лицензии CC BY-SA; `cargo xtask font-glyphs` генерирует `glyphs.txt` из TTF.
- [ ] `cargo xtask charset-check`: все строки в `godot/src/**/*.gd` и `data/**` состоят из
  глифов `glyphs.txt`; таблица нормализации ▶ → ►; в дизайн-книге ▶ заменить на ►.
- [ ] `cargo xtask etalons`: прочитать inline-скрипт дизайн-книги, исполнить его в
  `boa_engine` после прелюдии `crates/xtask/src/etalons/dom_shim.js`, собрать `innerHTML`
  шести `scr-*` и палитру из CSS, разобрать span'ы в сетки `design/etalons/<screen>.screen`
  и `palette.txt`; флаг `--check` сравнивает с закоммиченными файлами и с `ega.gd`.
- [ ] `godot/src/screens/boot.gd`: заставка один в один с макетом; `godot/tools/dump_screen.gd`
  рисует экран по имени (и фикстуре `design/etalons/<screen>.view.json`, если есть) и пишет
  буфер в `.tools/out/<screen>.screen`; `cargo xtask screen-check <screen>` сравнивает с
  эталоном и печатает первую расходящуюся строку.
- [ ] `cargo xtask screenshot <screen>`: оконный запуск, PNG в `.tools/out/`; в CI на Linux
  под Xvfb; документировать предел безголового режима в `docs/CONTENT.md` разделом «Экраны».

**Done when (EXECUTABLE):**
```
cargo xtask etalons --check
```
- Шесть эталонов и палитра извлечены из дизайн-книги движком Boa и совпадают с закоммиченными (exit 0).

**Done when (EXECUTABLE):**
```
cargo xtask screen-check boot
```
- Буфер заставки совпадает с эталоном макета посимвольно и по цветам.

**Done when (EXECUTABLE):**
```
cargo xtask charset-check
```
- Все строки интерфейса из набора глифов шрифта.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript узла и заставки проходит проверку парсера Godot.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Новые команды xtask без предупреждений.

**Done when (AGENT-JUDGED):** PNG заставки (`cargo xtask screenshot boot`) совпадает с макетом «Заставка» глазами: шрифт 9×16 без сглаживания, пропорции 4:3, цвета EGA, курсор в последней строке. Обоснование agent-judged: пиксели безголовый Godot не рендерит, буфер уже проверен машинно.

### Phase 3: bridge-contract — контракт состояния и команд, минимальный cdylib, кадр рынка

- [ ] Основание: docs/SPEKULYANT.md §14.1 (мост), §14.4, §13.3 (все одиннадцать экранов — их
  поля задают форму StateView); design/design-book.html §XIV (поток «состояние наружу,
  команды внутрь»), макет «Рынок». Потребляет из `text-screen`: узел и `screen-check`;
  из `bootstrap`: крейты.
- [ ] `crates/speculant-core/src/view/`: `StateView` (шапка: город, дата, строки денег и
  курсов; рынок: строки товаров с ценами покупки/продажи и запасом; карман; слухи; лента
  сообщений; время: скорость, пауза; ожидающее событие: текст, выборы, исход; пейджер;
  список дел и экран управляющего; связи; деньги по местам хранения; записная книжка; зона;
  итоги жизни; выбор биографии) — все строки готовы к показу; `Command` (NewLife, Advance,
  SetSpeed, Pause, Buy, Sell, Travel, Choose, OpenScreen, SetPolicy, Hire, Fire, Call,
  Deposit, Withdraw, Save, Load, Quit); `CommandError`; фасад `Game` с заглушкой, отдающей
  фикстуру `design/etalons/market.view.json`.
- [ ] `crates/speculant-godot/src/lib.rs`: `#[gdextension]`, класс `SpeculantGame`
  (RefCounted) с методами `new_life(json)`, `advance(delta_ms) -> bool`,
  `apply(json) -> json`, `view_json() -> String`; конвертации покрыты юнит-тестами крейта.
- [ ] `cargo xtask build-godot [--release] [--universal]`: сборка cdylib в `godot/bin/<platform>/`
  (на macOS — lipo arm64 + x86_64); `cargo xtask bridge-smoke`: сборка, безголовый запуск
  `godot/tools/bridge_smoke.gd` — создать `SpeculantGame`, разобрать `view_json()`, проверить
  поле города и восемь строк рынка.
- [ ] `godot/src/screens/market.gd`: экран рынка по макету из `StateView` (шапка, окно
  толкучки с выделенной строкой, «Карман» с индикатором ОБХСС, «Слухи», «Сообщения»,
  индикатор времени, строка F-клавиш); `cargo xtask screen-check market` на фикстуре.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test view
```
- StateView и Command проходят serde туда-обратно, фикстура рынка загружается (exit 0).

**Done when (EXECUTABLE):**
```
cargo xtask bridge-smoke
```
- cdylib собирается, Godot безголово создаёт класс моста и читает состояние.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check market
```
- Экран рынка на фикстуре совпадает с эталоном макета.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-godot
```
- Конвертации моста покрыты и зелёные.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript-typecheck фазы.

### Phase 4: core-time — время, календарь эпох, детерминированный RNG, скорость и пауза

- [ ] Основание: docs/SPEKULYANT.md §1 (опора 1), §3 (границы эпох, §3.6), §11 (7300
  дней); design/design-book.html §III, индикатор времени на макете рынка. Потребляет из
  `bridge-contract`: фасад `Game` (заглушка тика заменяется настоящей).
- [ ] Решение 15.3 с пользователем (варианты в Knowledge) → `docs/SPEKULYANT.md` §15
  `**Решено** (DECIDED-15-3)` и §13.1; параметры скоростей в `data/epochs/time.toml`.
- [ ] `src/time/`: `GameDate` (индекс дня от 1984-01-01, календарь, русские месяцы и дни
  недели, возраст персонажа), `EpochTable` (границы эпох из данных), `Clock` (скорости 1–3,
  пауза, перемотка до события или прибытия, `advance(ms) -> Vec<DayTick>`).
- [ ] `src/rng/`: `WorldRng` на ChaCha8 с именованными подпотоками (`stream("prices")`,
  `stream("events")` …), чтобы новые вызовы в одной подсистеме не сдвигали другие.
- [ ] `Game::advance` и команды `SetSpeed` / `Pause` настоящие; `StateView.header.date` и
  `time` из ядра.
- [ ] `tests/determinism.rs`: две игры с одним сидом и одним сценарием команд дают
  побайтно равный `view_json` после 400 дней; тест растёт с каждой подсистемой.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-3 docs/SPEKULYANT.md
```
- Масштаб времени принят с пользователем и внесён в документ.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test time
```
- Календарь, эпохи, скорости, перемотка и подпотоки RNG по спецификации тестов (exit 0).

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test determinism
```
- Один сид — одна история: проверено на 400 днях.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 5: content-format — формат данных, загрузчик, схема, валидатор ссылок, авторинг

- [ ] Основание: docs/SPEKULYANT.md §10.1, §10.3, §14.1 (контент как данные), §14.3
  (валидатор); design/design-book.html §X (таблица «Из чего состоит событие»), §XIV.
  Потребляет из `core-time`: `GameDate` и `EpochTable` для окон дат.
- [ ] Решение 15.7 с пользователем → `DECIDED-15-7` в §15 и §10/§14.
- [ ] `src/content/`: типы `Good` (категория, кривая опорных точек), `City` (множители по
  категориям, доступность по эпохам), `Route`, `Epoch`, `Biography`, `NpcTemplate`, `Event`
  (условия структурными таблицами: эпоха, окно дат, город, репутация, товар/деньги, флаги,
  NPC, узел таймлайна; вес; текст с подстановками; выборы с условиями; последствия;
  отложенные эффекты), `ContentId`-newtype'ы, `Content::load(dir)` с ошибками «файл:строка».
- [ ] Раскладка `data/`: `goods/`, `cities/`, `routes.toml`, `epochs/`, `biographies/`,
  `npc/`, `events/<epoch>/`, `history/`, `zone/events/`; минимальный валидный набор для
  тестов — `crates/speculant-core/tests/fixtures/content-min/` (два города, три товара,
  одна эпоха, одно событие с цепочкой, одна биография) и набор с намеренными ошибками.
- [ ] `speculant-validate <dir>`: схема, битые ссылки на товары / города / NPC / флаги /
  события, выборы без последствий, окна дат вне 1984–2004+; `--watch` через notify; вывод
  «файл:строка: сообщение».
- [ ] `cargo xtask new-event <epoch> <slug>` — заготовка события из шаблона;
  `docs/CONTENT.md` — справочник формата с примером события из §10.2.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-7 docs/SPEKULYANT.md
```
- Формат и инструмент авторинга приняты с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test content
```
- Загрузчик читает минимальный набор, отвергает набор с ошибками по каждому виду (exit 0).

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- crates/speculant-core/tests/fixtures/content-min
```
- Валидатор как CLI зелёный на валидном наборе.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 6: core-economy — кривые, города, запас и спрос, индекс инфляции, валюты, хранение

- [ ] Основание: docs/SPEKULYANT.md §5.1–§5.3, §6, §3 (деньги по эпохам);
  design/design-book.html §V (график: реальные единицы и номинал, таблица индекса), §VI.
  Потребляет из `content-format`: типы товаров, городов, эпох; из `core-time`: даты.
- [ ] `src/economy/`: `Real` (i64, копейки 1984 года), `PriceCurve` (лог-интерполяция между
  опорными точками через `libm`), `CityMarket` (запас, спрос, спред покупки/продажи,
  просадка от крупной сделки и восстановление по дням), `InflationIndex` (таблица по годам,
  дневная интерполяция, деноминация 1998-го как узел индекса), `Currency` (рубль СССР,
  чеки, доллар, новый рубль) и `format_money`, `MoneyStore` (матрас, сберкасса, чеки,
  доллары, коммерческий банк, ваучеры, пирамиды, ГКО — окна доступности и профиль риска
  для событий), `apply_shock(good, city, factor, days)`.
- [ ] Команды `Buy`, `Sell`, `Deposit`, `Withdraw` настоящие; `StateView` рынка, кармана и
  денег из ядра.
- [ ] Тесты на фикстурах из графика дизайн-книги: видеомагнитофон 1986 против 2000, три
  нуля при деноминации, обвал цены после крупной продажи и восстановление.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test economy
```
- Кривые, множители, запас/спрос, индекс, валюты и хранение по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test determinism
```
- Детерминизм сохранён с экономикой.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 7: core-reputation — фракции, связи, репутация по эпохам

- [ ] Основание: docs/SPEKULYANT.md §9, §3 (угрозы по эпохам); design/design-book.html §IX.
  Потребляет из `content-format`: шаблоны NPC как источник связей; из `core-time`: эпохи.
- [ ] Решение 15.6 с пользователем → `DECIDED-15-6` в §15 и §9.
- [ ] `src/reputation/`: `Faction`, `Reputation` (целые уровни с дрейфом к нулю по эпохе),
  `Contact` (именованный человек: роль, фракция, долг, история, доступность), операции
  «попросить / задолжать / потерять», видимость фракций по эпохам.
- [ ] `StateView.contacts` и индикаторы фракций (например, ОБХСС в «Кармане») из ядра.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-6 docs/SPEKULYANT.md
```
- Набор фракций принят с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test reputation
```
- Фракции, связи и их динамика по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 8: core-events — движок событий

- [ ] Основание: docs/SPEKULYANT.md §10.1, §10.2, §3.6; design/design-book.html §X, макет
  «Событие» (окно, выборы 1–N, исход). Потребляет из `content-format`: схему события; из
  `core-economy`: деньги, товар, шоки; из `core-reputation`: условия и последствия по
  фракциям и связям; из `core-time`: окна дат и отложенные эффекты.
- [ ] `src/events/`: вычисление условий над состоянием мира, взвешенный выбор дня (кулдауны,
  однократные события, флаги), доступность выборов, последствия (деньги, товар, репутация,
  связи, флаги, сдвиг цен), отложенные эффекты с вероятностью, цепочки, подстановки текста
  с номиналом текущей валюты, журнал событий жизни.
- [ ] Команда `Choose` настоящая; `StateView.pending_event` и экран исхода; время
  останавливается на событии.
- [ ] Тест на примере §10.2: три исхода дают ровно описанные последствия и флаг.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test events
```
- Условия, веса, выборы, последствия, отложенные эффекты и цепочки по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test determinism
```
- Детерминизм сохранён с событиями.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 9: core-travel — рейсы, направления по эпохам, дорога как событие

- [ ] Основание: docs/SPEKULYANT.md §4, §13.3 (экран «Поездка»); design/design-book.html §IV.
  Потребляет из `core-events`: события в дороге; из `core-economy`: расходы и таможня.
- [ ] `src/travel/`: маршруты из `routes.toml`, доступность по эпохе и по блату, стоимость и
  время в пути, прибытие как узел перемотки, дорожные события по весам маршрута.
- [ ] Команда `Travel` настоящая; `StateView.travel` (направления, цена, дни).

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test travel
```
- Доступность направлений по эпохам, стоимость, время и дорожные события по тестам.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 10: core-npc — соперники с памятью

- [ ] Основание: docs/SPEKULYANT.md §8, §5.2 (пункт 5); design/design-book.html §VIII.
  Потребляет из `core-economy`: торговля по тем же правилам; из `core-reputation`: связи и
  обиды; из `core-events`: торг через события.
- [ ] `src/npc/`: выбор 10–15 персонажей из шаблонов по сиду, стратегии (торговец, челнок,
  рэкетир, чиновник, банкир …), грубая симуляция по дням (сделки, запас городов, открытие
  дел), память отношений (партнёрство, кредит, обида с возвратом через полгода), судьбы
  (зона, разорение, гибель, депутат).
- [ ] `StateView.contacts` расширен соперниками; события получают условия «участвует NPC».

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test npc
```
- Состав по сиду, стратегии, влияние на цены и память отношений по тестам.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test determinism
```
- Детерминизм сохранён с NPC.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 11: core-business — дела, сотрудники, управляющие, политики, прогноз

- [ ] Основание: docs/SPEKULYANT.md §7.1–§7.3, §13.3 (экран управляющего);
  design/design-book.html §VII, макет «Управляющий» (шесть политик, черты, прогноз).
  Потребляет из `core-economy`: запасы и выручка; из `core-npc`: сотрудники из шаблонов;
  из `core-events`: события «вне политик».
- [ ] `src/business/`: формы по эпохам (цех → кооператив → ТОО → ООО → банк), типы дел
  (ларьки, магазины, производство, челночный рейс, банк), `Employee` (навык, честность,
  лояльность, привычки), `Manager` и `Policy` (наценка, крыша, закупка, ночная торговля,
  поведение при проверке, зарплата), месячный прогноз (выручка, риск наезда, риск
  воровства), генерация событий только при выходе за политики.
- [ ] Команды `SetPolicy`, `Hire`, `Fire` настоящие; `StateView.businesses` и `manager`.
- [ ] Тест прогноза на фикстуре макета: положения политик дают те же уровни, что макет.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test business
```
- Дела, сотрудники, политики и прогноз по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 12: core-life — персонаж, биографии, быт, финалы, итоги

- [ ] Основание: docs/SPEKULYANT.md §11, §11.1, §11.2, §13.3 (экраны итогов и выбора
  биографии); design/design-book.html §XI, макет «Итоги жизни». Потребляет из
  `core-events`: финалы приходят последствиями; из `core-business`: дела в итогах.
- [ ] Решения 15.4 и 15.5 с пользователем → `DECIDED-15-4`, `DECIDED-15-5` в §15 и §11.
- [ ] `src/life/`: `Character` (возраст, биография, быт по решению 15.4), старт жизни из
  `data/biographies/`, финалы (смерть, зона, разорение с долгами, уход) с текстом в духе
  эпохи, `LifeSummary` (капитал, лучшая и худшая сделки, срок, открытия).
- [ ] Команда `NewLife` настоящая; `StateView.biographies`, `StateView.summary`.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-4 docs/SPEKULYANT.md
```
- Быт персонажа решён с пользователем.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-5 docs/SPEKULYANT.md
```
- Список биографий и условия открытия решены с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test life
```
- Старт по биографии, старение, финалы и итоги по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 13: core-zone — режим зоны

- [ ] Основание: docs/SPEKULYANT.md §12, §13.3 (экраны зоны); design/design-book.html §XII.
  Потребляет из `core-life`: финал «зона» и выход; из `core-business`: доверенные лица на
  воле; из `core-events`: события зоны из `data/zone/events/`.
- [ ] Решение 15.8 с пользователем → `DECIDED-15-8` в §15 и §12.
- [ ] `src/zone/`: `Sentence`, ускоренный тик, своя экономика (сигареты, чай, передачи,
  промзона, услуги), свой авторитет, поведение доверенных лиц, выход (срок, УДО, амнистия
  как узел таймлайна, взятка), последствия (связи «оттуда», открытие биографии).
- [ ] `StateView.zone`; переход рынок ↔ зона в `Game`.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-8 docs/SPEKULYANT.md
```
- Механика зоны решена с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test zone
```
- Срок, экономика, авторитет, доверенные лица и выход по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 14: core-history — генератор истории по сиду и генератор после 2004-го

- [ ] Основание: docs/SPEKULYANT.md §3.1–§3.6, §3.5 (песочница); design/design-book.html
  §III. Потребляет из `core-events`: исторические события как события с условием таймлайна;
  из `core-economy`: шоки и узлы индекса; из `core-time`: окна дат.
- [ ] Решение 15.9 с пользователем → `DECIDED-15-9` в §15 и §3.5.
- [ ] `src/history/`: `Timeline` по сиду из `data/history/` (окно, масштаб, альтернативы:
  сухой закон, павловский обмен, ГКЧП, распад, либерализация, МММ, чёрный вторник,
  деноминация, дефолт, плоский налог …), переходы эпох через события; генератор после
  2004-го по решению 15.9 из `data/history/post2004/`.
- [ ] Тесты: два сида дают разные даты в пределах окон; каркас эпох неизменен; после
  2004-го генератор выдаёт события с заданной частотой.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-9 docs/SPEKULYANT.md
```
- Генератор после 2004-го решён с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test history
```
- Плавающие даты, масштаб, альтернативы и песочница по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test determinism
```
- Детерминизм сохранён с таймлайном.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 15: core-meta — записная книжка, открытия, слухи, рекорды, сохранения

- [ ] Основание: docs/SPEKULYANT.md §11.3, §11.4, §13.3 (записная книжка), §14.1
  (сериализация); design/design-book.html §XI. Потребляет из `core-life`: итоги жизни как
  источник открытий; из всего ядра: состояние мира для сохранений.
- [ ] Решение 15.10 с пользователем → `DECIDED-15-10` в §15 и §11.4.
- [ ] `src/meta/`: `Notebook` (товары, люди, схемы), открытия биографий, стартовые слухи,
  `Records` (раздельно ironman и свободный режим); `src/save/`: версионированный формат
  жизни и отдельный файл метапрогрессии, ironman-правила (один слот, автосохранение,
  удаление в финале), тест миграции формата с версии 1.
- [ ] Команды `Save` / `Load` настоящие; `StateView.notebook`, `records`.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-10 docs/SPEKULYANT.md
```
- Таблица рекордов решена с пользователем.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test meta
```
- Книжка, открытия, слухи и рекорды по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-core --test save
```
- Сохранение и загрузка дают побайтно тот же `view_json`; ironman-правила соблюдены.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 16: content-world — товары, города, маршруты, эпохи, биографии, шаблоны NPC

- [ ] Основание: docs/SPEKULYANT.md §3, §4, §5.3, §6, §8, §11.1; design/design-book.html
  §III–§VI, §VIII, §XI, график цен. Потребляет из `content-format`: формат и валидатор;
  относительно `core-meta` — `order-only`.
- [ ] Решения 15.1, 15.2 и 15.11 с пользователем: вопрос 15.11 (объём контента) добавить в
  §15 документа одиннадцатым пунктом, затем `DECIDED-15-1`, `DECIDED-15-2`, `DECIDED-15-11`
  в §15, §4, §5.3, §10.3; принятые числа — в `data/balance.toml` `[volume]`.
- [ ] `data/goods/` — товары по решению 15.2 с опорными точками по годам и категориями;
  `data/cities/` — города по решению 15.1 с множителями; `data/routes.toml` по таблице §4;
  `data/epochs/` — пять эпох с законами, валютами, угрозами и видимостью фракций;
  `data/biographies/` по решению 15.5; `data/npc/` — пул шаблонов по решению 15.11.
- [ ] `cargo xtask content-count <kind>`: считает сущности вида (`goods`, `cities`, `npc`,
  `events:<epoch>`, `zone-events`, `post2004-templates`) и сверяет с `[volume]` из
  `data/balance.toml`; `docs/CONTENT.md` дополнен разделами по каждому виду данных.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-1 docs/SPEKULYANT.md
```
- Города и множители решены с пользователем.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-2 docs/SPEKULYANT.md
```
- Ассортимент решён с пользователем.

**Done when (EXECUTABLE):**
```
grep -qF DECIDED-15-11 docs/SPEKULYANT.md
```
- Объём контента решён с пользователем и записан в `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Весь контент мира проходит схему и проверку ссылок (exit 0).

**Done when (EXECUTABLE):**
```
cargo xtask content-count goods
```
- Ассортимент не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask content-count npc
```
- Пул шаблонов соперников не ниже порога из `balance.toml`.

### Phase 17: simulator — боты, тысячи жизней, метрики, калибровка порогов

- [ ] Основание: docs/SPEKULYANT.md §14.1 (пункт 4), §14.3 (пункт 1); design/design-book.html
  §XIV («Симулятор»). Потребляет из `speculant-core` целиком через фасад `Game` (настоящий
  крейт, без моков); из `content-world`: данные `data/`.
- [ ] `crates/speculant-sim/`: стратегии ботов из Knowledge, параллельный прогон
  (`--lives`, `--seed`, `--strategy`, `--threads`), метрики (выживание по годам, капитал,
  финалы, покрытие событий, дни без событий, стратегии выше N медиан), отчёт JSON и текст,
  экспорт покрытия для валидатора.
- [ ] Калибровка: прогнать базу на контенте мира, показать пользователю измеренные метрики и
  предложенные от них пороги, по цифре записать в `data/balance.toml` секции `[balance]` и
  `[coverage]` со строкой `# calibrated: seed, lives, date`.
- [ ] `cargo xtask balance-check <epoch|all>`: прогон и сверка с `[balance]`; CI гоняет 200
  жизней на каждом пуше.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-sim
```
- Стратегии и метрики по спецификации тестов.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-sim -- --lives 200 --seed 1
```
- Двести жизней на настоящем ядре и данных проходят без паники и дают отчёт (exit 0).

**Done when (EXECUTABLE):**
```
grep -qF calibrated: data/balance.toml
```
- Пороги баланса и покрытия подтверждены пользователем от измеренной базы, не назначены планом.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

### Phase 18: content-events-1984 — события застоя

- [ ] Основание: docs/SPEKULYANT.md §3.1, §10.2, §6 (чеки, доллары под полом), §9;
  design/design-book.html §III (1984–87), макет «Событие». Потребляет из `simulator`:
  балансовый прогон; из `content-format`: `new-event` и валидатор.
- [ ] `data/events/stagnation/` — события застоя в объёме решения 15.11: фарцовка,
  «Берёзка», моряки, ОБХСС, участковый, соседи, дружинники, очереди, «выбросили»,
  антиалкогольная кампания, блат; цепочки с отложенными эффектами; событие §10.2 дословно.
- [ ] `data/history/` — узлы 1984–1987 с окнами и альтернативами.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Все события застоя проходят схему и ссылки.

**Done when (EXECUTABLE):**
```
cargo xtask content-count events:stagnation
```
- Объём эпохи не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask balance-check stagnation
```
- Прогон жизней 1984–1987 в откалиброванных порогах (смертность, скука, ломающие схемы).

### Phase 19: content-events-1988 — события перестройки

- [ ] Основание: docs/SPEKULYANT.md §3.2, §7.1 (кооператив), §6 (сберкасса, банк);
  design/design-book.html §III (1988–91). Потребляет из `content-events-1984`: флаги и
  цепочки, переходящие через 1988-й; `order-only` относительно симулятора.
- [ ] `data/events/perestroika/` — события перестройки в объёме решения 15.11: кооперативы,
  видеосалоны, варёнки, биржи, компьютеры, первый рэкет, исполкомы, ревизии, павловский
  обмен, сухой закон и сахар, ГКЧП, распад.
- [ ] `data/history/` — узлы 1988–1991.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Контент перестройки валиден.

**Done when (EXECUTABLE):**
```
cargo xtask content-count events:perestroika
```
- Объём эпохи не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask balance-check perestroika
```
- Прогон 1984–1991 в откалиброванных порогах.

### Phase 20: content-events-1992 — события дикого рынка

- [ ] Основание: docs/SPEKULYANT.md §3.3, §4 (Польша, Турция, Китай), §6 (ваучеры,
  пирамиды, ГКО), §7; design/design-book.html §III (1992–98), макеты «Управляющий» и
  «Пейджер». Потребляет из `content-events-1988`: переходящие цепочки.
- [ ] `data/events/wild_market/` — события дикого рынка в объёме решения 15.11: челноки,
  ларьки, крыша, бандиты, налоговая полиция, приватизация, ваучеры, МММ, чёрный вторник,
  свой банк, лопнувший банк, кинувший партнёр, деноминация, дефолт; события пейджера.
- [ ] `data/history/` — узлы 1992–1998.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Контент дикого рынка валиден.

**Done when (EXECUTABLE):**
```
cargo xtask content-count events:wild_market
```
- Объём эпохи не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask balance-check wild_market
```
- Прогон 1984–1998 в откалиброванных порогах.

### Phase 21: content-events-1999 — события стабилизации

- [ ] Основание: docs/SPEKULYANT.md §3.4, §7.1 (холдинг, банк), §9 (силовики);
  design/design-book.html §III (1999–04). Потребляет из `content-events-1992`: цепочки.
- [ ] `data/events/stabilization/` — события стабилизации в объёме решения 15.11: силовики,
  рейдеры, проверки, плоский налог, сетевая розница, сотовая связь, интернет, производство,
  гипермаркеты, соперники в депутатах, «мобила-кирпич».
- [ ] `data/history/` — узлы 1999–2004.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Контент стабилизации валиден.

**Done when (EXECUTABLE):**
```
cargo xtask content-count events:stabilization
```
- Объём эпохи не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask balance-check all
```
- Полные жизни 1984–2004 в откалиброванных порогах.

### Phase 22: content-zone-post2004 — события зоны и шаблоны песочницы

- [ ] Основание: docs/SPEKULYANT.md §12, §3.5; design/design-book.html §XII, §III (2005+).
  Потребляет из `core-zone` и `core-history`: форматы событий зоны и шаблонов; из
  `content-events-1999`: `order-only`.
- [ ] `data/zone/events/` — события зоны в объёме решения 15.11 (советская и зона 90-х
  отдельно); `data/history/post2004/` — параметрические шаблоны по решению 15.9 в объёме
  решения 15.11.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data
```
- Контент зоны и песочницы валиден.

**Done when (EXECUTABLE):**
```
cargo xtask content-count zone-events
```
- Объём зоны не ниже порога из `balance.toml`.

**Done when (EXECUTABLE):**
```
cargo xtask content-count post2004-templates
```
- Шаблонов песочницы не ниже порога из `balance.toml`.

### Phase 23: validator-full — недостижимые и мёртвые события, покрытие по прогону

- [ ] Основание: docs/SPEKULYANT.md §14.1 (пункт 5), §14.3 (пункт 2);
  design/design-book.html §XIV («Валидатор»). Потребляет из `simulator`: экспорт покрытия;
  из `content-*`: полный набор данных.
- [ ] `speculant-validate --strict`: условия, невыполнимые статически (окна вне таймлайна,
  флаги без источника, NPC без шаблона, цепочки на события с несовместимыми условиями);
  `--coverage <sim.json>`: события без срабатываний за `[coverage] lives` жизней — ошибка
  без `rare`.
- [ ] `cargo xtask coverage-check`: прогон `[coverage] lives` жизней и строгая валидация
  одной командой.

**Done when (EXECUTABLE):**
```
cargo test -p speculant-validate
```
- Каждый вид дефекта ловится на негативной фикстуре.

**Done when (EXECUTABLE):**
```
cargo run -p speculant-validate -- data --strict
```
- Весь контент без недостижимых событий и мёртвых цепочек.

**Done when (EXECUTABLE):**
```
cargo xtask coverage-check
```
- Каждое событие хотя бы раз сработало за откалиброванное число жизней или помечено редким.

### Phase 24: bridge-full — все команды и виды, сохранения, пейджер

- [ ] Основание: docs/SPEKULYANT.md §14.1 (пункт 2), §14.2, §13.3; design/design-book.html
  §XIV. Потребляет из `speculant-core`: полный `Game` (настоящий крейт); из
  `bridge-contract`: четыре метода.
- [ ] `crates/speculant-godot/`: все `Command`'ы через `apply`, ошибки команд как JSON,
  `advance` возвращает «состояние изменилось», байты сохранений наружу и внутрь;
  `godot/src/saves/` — запись в `user://saves/` и `user://meta.spk`.
- [ ] `cargo xtask bridge-smoke --full`: новая жизнь, 30 дней, покупка, продажа, событие с
  выбором, поездка, сохранение и загрузка через мост безголово.

**Done when (EXECUTABLE):**
```
cargo xtask bridge-smoke --full
```
- Мост гоняет настоящее ядро по всем командам (exit 0).

**Done when (EXECUTABLE):**
```
cargo test -p speculant-godot
```
- Конвертации всех команд покрыты.

**Done when (EXECUTABLE):**
```
cargo clippy --workspace --all-targets -- -D warnings
```
- Rust-typecheck фазы.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript-typecheck фазы.

### Phase 25: ui-market-event — рынок вживую, событие, поездка, ввод, время

- [ ] Основание: docs/SPEKULYANT.md §13.1 (пункт 4), §13.3 (пункты 1–3), §1 (опора 1);
  design/design-book.html макеты «Рынок» и «Событие», §XIII. Потребляет из `bridge-full`:
  живое состояние; из `text-screen`: узел.
- [ ] `godot/src/input/keymap.gd`: F1–F10, стрелки, цифры, пробел, +/−, Esc, Enter, мышь по
  строке F-клавиш и строкам окон; `godot/src/screens/router.gd` — стек экранов.
- [ ] `market.gd` на живом состоянии (выделение, покупка/продажа диалогом количества в окне,
  скорости и пауза), `event.gd` — красное модальное окно с тенью поверх текущего экрана,
  выборы цифрами, экран исхода; `travel.gd` — направления, цена, дни, события в дороге.
- [ ] `godot/tools/ui_flow.gd` + `cargo xtask ui-flow-smoke`: безголово — рынок → F2 покупка
  → пробел пауза → событие → выбор 1 → исход → F4 поездка → прибытие; проверка буферов.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check event
```
- Экран события на фикстуре совпадает с эталоном макета.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check market
```
- Рынок по-прежнему совпадает с эталоном после перевода на живое состояние.

**Done when (EXECUTABLE):**
```
cargo xtask ui-flow-smoke
```
- Клавиатурный сценарий рынок → событие → поездка проходит безголово.

**Done when (EXECUTABLE):**
```
cargo xtask screen-smoke travel
```
- Экран поездки рисуется без ошибок, строка F-клавиш на месте.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript-typecheck фазы.

**Done when (AGENT-JUDGED):** PNG рынка в трёх скоростях и на паузе и PNG события с тремя исходами совпадают с макетами глазами; экран поездки держит правила §XIII (синие окна, тени, псевдографика). Обоснование agent-judged: состояния и пиксели вне буферной сверки.

### Phase 26: ui-screens — дела и управляющий, связи, деньги, пейджер, книжка, зона, итоги, старт

- [ ] Основание: docs/SPEKULYANT.md §13.3 (пункты 4–11), §7.3, §11.2, §12;
  design/design-book.html макеты «Управляющий», «Пейджер», «Итоги жизни», §XIII. Потребляет
  из `ui-market-event`: роутер и ввод; из `bridge-full`: виды.
- [ ] `godot/src/screens/`: `businesses.gd` и `manager.gd` (политики ◄ ►, черты, прогноз),
  `contacts.gd`, `money.gd`, `pager.gd` (лента, листание, ответ), `notebook.gd`, `zone.gd`
  (набор экранов режима), `summary.gd` (некролог / приговор / заметка), `new_life.gd`
  (биография, режим ironman / свободный, сид).
- [ ] Главное меню и меню сохранений кодом; выход из игры с подтверждением в духе
  `Abort, Retry, Fail?`.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check manager
```
- Экран управляющего на фикстуре совпадает с эталоном.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check pager
```
- Пейджер совпадает с эталоном.

**Done when (EXECUTABLE):**
```
cargo xtask screen-check end
```
- Итоги жизни совпадают с эталоном.

**Done when (EXECUTABLE):**
```
cargo xtask screen-smoke all
```
- Все экраны без макета рисуются без ошибок со строкой F-клавиш.

**Done when (EXECUTABLE):**
```
cargo xtask charset-check
```
- Новые строки из набора глифов.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript-typecheck фазы.

**Done when (AGENT-JUDGED):** экраны без макета (связи, деньги, книжка, зона, выбор биографии, меню) по PNG держат правила §XIII дизайн-книги. Обоснование agent-judged: эталонов для них нет, это приёмка стиля.

### Phase 27: ui-sound-crt — PC-speaker, мелодия заставки, CRT-эффект

- [ ] Основание: docs/SPEKULYANT.md §13.1 (пункты 5–6), §13.2; design/design-book.html
  §XIII («Звук»). Потребляет из `ui-screens`: события интерфейса как источники бипов.
- [ ] `godot/src/sound/speaker.gd`: прямоугольная волна через `AudioStreamGenerator`, бипы
  событий, пейджера, ошибок; мелодия заставки нотами из `data/sound/boot.toml`;
  `godot/tools/sound_smoke.gd` проверяет частоту заполненного буфера.
- [ ] `godot/src/crt/crt.gdshader` + переключатель в настройках (по умолчанию выключен).

**Done when (EXECUTABLE):**
```
cargo xtask sound-smoke
```
- Генератор выдаёт ноту заданной частоты в безголовом режиме.

**Done when (EXECUTABLE):**
```
cargo xtask gd-check
```
- GDScript-typecheck фазы.

**Done when (AGENT-JUDGED):** мелодия заставки и бипы звучат как PC-speaker, CRT-эффект включается и выключается без потери читаемости. Обоснование agent-judged: звук и шейдер не проверяются буфером.

### Phase 28: release — экспорт под три платформы, релизный CI, версия

- [ ] Основание: docs/SPEKULYANT.md §14.4, §2 (платформы); design/design-book.html §XIV
  («Цена решения»). Потребляет из `bridge-contract`: `build-godot --universal`; из
  `bootstrap`: CI.
- [ ] `godot/export_presets.cfg`: Windows x86_64, Linux x86_64, macOS universal;
  `cargo xtask godot-fetch --templates`; `cargo xtask export <platform>` в `.tools/dist/`;
  `cargo xtask export-check` — пресеты валидны, экспорт хост-платформы собирается и
  запускается с `--headless --quit`.
- [ ] `cargo xtask version-check`: `Cargo.toml` workspace, `project.godot`
  `application/config/version` и тег релиза согласованы; `.github/workflows/release.yml` по
  тегу `v*` собирает три платформы на родных раннерах и прикладывает артефакты.

**Done when (EXECUTABLE):**
```
cargo xtask export-check
```
- Экспорт хост-платформы собирается и запускается.

**Done when (EXECUTABLE):**
```
cargo xtask version-check
```
- Версия в одном месте и согласована.

**Done when (EXECUTABLE):**
```
grep -qF export-check .github/workflows/release.yml
```
- Релизный workflow гоняет тот же гейт, что и локально.

**Done when (AGENT-JUDGED):** логика `release.yml` согласована: version-check стоит до сборки, артефакты трёх платформ прикладываются к релизу, macOS-библиотека universal. Обоснование agent-judged: исполнение на стороне GitHub локально невоспроизводимо.

### Phase 29: wire — сквозная жизнь на настоящих модулях

- [ ] Основание: docs/SPEKULYANT.md целиком; design/design-book.html §XIV (игровой поток).
  Потребляет ВСЕ кодовые фазы: настоящие ядро, мост, экраны, контент.
- [ ] `godot/tools/e2e_life.gd` + `cargo xtask e2e`: сборка cdylib, безголово — новая жизнь
  (сид 7, студент, ironman) → 60 дней с покупками и продажами → событие с выбором → поездка
  → сохранение и загрузка → продолжение до финала → экран итогов; проверка буферов и
  `view_json` на каждом шаге; моки только на границе файловой системы (`user://` во
  временную папку).
- [ ] Честный предел: пиксели, звук и окно не проверяются безголово — граница этой фазы
  объявлена точкой приёмки человеком по PNG и запуску экспорта.
- [ ] `docs/SPEKULYANT.md` §15 без нерешённых вопросов (все одиннадцать с маркерами),
  дизайн-книга §XV с теми же маркерами.

**Done when (EXECUTABLE):**
```
cargo xtask e2e
```
- Полная жизнь проходит безголово на настоящих модулях (exit 0).

**Done when (EXECUTABLE):**
```
cargo run -p speculant-sim -- --lives 50 --seed 7
```
- Настоящее ядро с полным контентом выдерживает прогон.

**Done when (EXECUTABLE):**
```
cargo test --workspace
```
- Вся рабочая область зелёная.

**Done when (EXECUTABLE):**
```
cargo xtask deps-check
```
- Ядро по-прежнему без Godot.

**Done when (EXECUTABLE):**
```
cargo xtask docs-check --all-decided
```
- Все одиннадцать решений §15 внесены в оба документа.
