extends SceneTree
## Безголовый клавиатурный сценарий для `cargo xtask ui-flow-smoke`: настоящее ядро через мост и
## роутер экранов в чистом каталоге игрока — заставка под мелодию «Коробейников» → главное меню →
## настройки: CRT включается и выключается → новая жизнь (студент, свободный режим, сид 7) → рынок:
## продажа того, чего нет, — бип отказа, F2 покупка в окне количества, пробел пауза и снова ход,
## событие с бипом → выбор → исход, F4 поездка → прибытие; дела, связи и пейджер — время в них
## стоит; меню выхода → сохранение в слот → «Abort, Retry, Fail?» → F в главное меню → загрузка
## слота → снова рынок того же дня; ironman-жизнь ложится в свой слот, «Продолжить» её возвращает,
## новая ironman-жизнь поверх спрашивает подтверждения; из главного меню A — выход из игры. Клавиши
## проходят через Keymap, как в игре; после шагов сверяется буфер экрана.

const SAVES_ROOT := "user://ui_flow"
const SETTINGS_FILE := "user://ui_flow/settings.cfg"
## Первая нота «Коробейников» — ми второй октавы.
const FIRST_MELODY_HERTZ := 659
const BIOGRAPHY := "student"
const SEED_KEY := KEY_7
const DAY_MILLISECONDS := 1800
const LONGEST_WAIT_STEPS := 2000
const STANDSTILL_DAYS := 5
const TRADE_GOOD := "chewing_gum"
const POCKET_LABEL := "Жвачка"
const DIGIT_KEYS := [KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9]
const CONTINUE_ITEM := 1
const NEW_LIFE_ITEM := 2
const LOAD_ITEM := 3
const SETTINGS_ITEM := 6
const SAVE_ITEM := 2
## Пункты меню выхода до добровольных уходов и после них.
const ITEMS_BEFORE_EXITS := 5
const ITEMS_AFTER_EXITS := 1

var _screen: TextScreen
var _router: Router
var _saves := Saves.new(SAVES_ROOT)
var _quit_asked := false
var _cues: Array[String] = []
var _melody: Array = []


func _initialize() -> void:
	_clear_saves()
	_screen = TextScreen.new()
	_router = Router.new(SpeculantGame.new(), _screen, _saves, Settings.new(SETTINGS_FILE))
	_router.quit_requested.connect(func() -> void: _quit_asked = true)
	_router.sound_cue.connect(func(cue_name: String) -> void: _cues.append(cue_name))
	_router.melody.connect(func(tones: Array) -> void: _melody = tones)
	_router.start()
	var passed := (
		_boot_and_menu()
		and _settings()
		and _new_life(true)
		and _refused_sale()
		and _buy()
		and _pause()
		and _event()
		and _travel()
		and _screens_stop_time()
		and _save_and_load()
		and _ironman()
		and _quit()
	)
	_clear_saves()
	_finish(passed)


func _boot_and_menu() -> bool:
	if _router.top_name() != "boot" or not _shows("SPEKUL.EXE"):
		return _fail("сеанс не начался с заставки:\n" + _buffer())
	if _melody.is_empty() or int(_melody[0]["hertz"]) != FIRST_MELODY_HERTZ:
		return _fail("заставка не отдала пищалке мелодию: %s" % [_melody])
	_press(KEY_ENTER)
	if _router.top_name() != "main_menu" or not _shows(" Главное меню "):
		return _fail("любая клавиша не открыла главное меню:\n" + _buffer())
	return true


## Цифра 6 — настройки; Enter включает CRT и выключает его снова, Esc — назад в меню.
func _settings() -> bool:
	_press(DIGIT_KEYS[SETTINGS_ITEM])
	if _router.top_name() != "settings" or not _shows(" Настройки "):
		return _fail("цифра %d не открыла настройки:\n%s" % [SETTINGS_ITEM, _buffer()])
	for expected: bool in [true, false]:
		_press(KEY_ENTER)
		if _router.view()["settings"][Settings.CRT] != expected or Settings.new(SETTINGS_FILE).enabled(Settings.CRT) != expected:
			return _fail("Enter не переключил CRT в %s:\n%s" % [expected, _buffer()])
	_press(KEY_ESCAPE)
	if _router.top_name() != "main_menu":
		return _fail("Esc из настроек не вернул меню:\n" + _buffer())
	return true


## F3 на товаре, которого нет в кармане, Enter — ядро отказывает, пищалка даёт бип отказа, Esc.
func _refused_sale() -> bool:
	_press(KEY_F3)
	_press(KEY_ENTER)
	if _cues.back() != Speaker.ERROR or not _shows("Количество:"):
		return _fail("продажа без товара не дала бип отказа: %s\n%s" % [_cues, _buffer()])
	_press(KEY_ESCAPE)
	return true


## Цифра 2 — новая жизнь; стрелками к студенту, F2 — свободный режим, цифра 7 — сид, Enter — жить.
func _new_life(free: bool) -> bool:
	_press(DIGIT_KEYS[NEW_LIFE_ITEM])
	if _router.top_name() != "new_life":
		return _fail("цифра %d не открыла новую жизнь:\n%s" % [NEW_LIFE_ITEM, _buffer()])
	var biographies: Array = _router.view()["biographies"]
	var row := biographies.find_custom(func(biography: Dictionary) -> bool: return biography["biography"] == BIOGRAPHY)
	for _step in row:
		_press(KEY_DOWN)
	if free:
		_press(KEY_F2)
		if not _shows("свободный"):
			return _fail("F2 не переключила режим:\n" + _buffer())
	_press(SEED_KEY)
	_press(KEY_ENTER)
	var view := _router.view()
	if _router.top_name() != "market" or view["ironman"] == free or view["header"]["city"] != "Москва":
		return _fail("жизнь студента не открыла рынок:\n" + _buffer())
	if not _shows("Купить") or not _shows("Время идёт"):
		return _fail("рынок не нарисован:\n" + _buffer())
	return true


## Стрелками к жвачке, F2, цифра 2, Enter — две пачки в кармане.
func _buy() -> bool:
	var goods: Array = _router.view()["market"]["goods"]
	var row := goods.find_custom(func(good: Dictionary) -> bool: return good["good"] == TRADE_GOOD)
	if row < 0:
		return _fail("жвачки нет на толкучке")
	for _step in row:
		_press(KEY_DOWN)
	_press(KEY_F2)
	if not _shows("Количество:"):
		return _fail("F2 не открыла окно количества:\n" + _buffer())
	_press(KEY_2)
	_press(KEY_ENTER)
	if _shows("Количество:"):
		return _fail("окно количества не закрылось после покупки:\n" + _buffer())
	if not _shows(POCKET_LABEL):
		return _fail("покупка не дошла до кармана:\n" + _buffer())
	return true


func _pause() -> bool:
	_press(KEY_SPACE)
	if not _shows("ПАУЗА"):
		return _fail("пробел не поставил паузу:\n" + _buffer())
	_press(KEY_SPACE)
	if not _shows("Время идёт"):
		return _fail("второй пробел не пустил время:\n" + _buffer())
	return true


## Время идёт до события; первый доступный выбор цифрой, исход, любая клавиша — снова рынок.
func _event() -> bool:
	for _step in LONGEST_WAIT_STEPS:
		if _router.top_name() == "event":
			break
		_router.tick(DAY_MILLISECONDS)
	if _router.top_name() != "event" or not _shows("СОБЫТИЕ") or not _shows("Время остановлено"):
		return _fail("за %d дней событие не пришло:\n%s" % [LONGEST_WAIT_STEPS, _buffer()])
	if _cues.back() != Speaker.EVENT:
		return _fail("событие пришло без бипа: %s" % [_cues])
	if not _settle_event():
		return false
	if _router.top_name() != "market":
		return _fail("после исхода не вернулся рынок:\n" + _buffer())
	return true


## F4 — направления, Enter на первом доступном, перемотка до прибытия; события дороги решаются.
func _travel() -> bool:
	_press(KEY_F4)
	if _router.top_name() != "travel" or not _shows(" Поездка "):
		return _fail("F4 не открыла поездку:\n" + _buffer())
	var destinations: Array = _router.view()["travel"]["destinations"]
	var row := destinations.find_custom(func(destination: Dictionary) -> bool: return destination["available"])
	if row < 0:
		return _fail("ни одного доступного направления:\n" + _buffer())
	for _step in row:
		_press(KEY_DOWN)
	var destination: String = destinations[row]["name"]
	_press(KEY_ENTER)
	if not _shows("В пути"):
		return _fail("Enter не отправил в путь:\n" + _buffer())
	_press(KEY_ENTER)
	for _step in LONGEST_WAIT_STEPS:
		if _router.top_name() == "market":
			break
		if _router.top_name() == "event":
			if not _settle_event():
				return false
		else:
			_router.tick(DAY_MILLISECONDS)
	var city: String = _router.view()["header"]["city"]
	if _router.top_name() != "market" or city != destination:
		return _fail("не доехали до %s, на экране %s:\n%s" % [destination, city, _buffer()])
	return true


## F6 — дела, F5 — связи, F7 — пейджер: экраны открываются, дни в них не идут, Esc — назад к рынку.
func _screens_stop_time() -> bool:
	for step: Array in [[KEY_F6, "businesses", " Дела "], [KEY_F5, "contacts", " Связи "], [KEY_F7, "pager", "ПЕЙДЖЕР"]]:
		_press(step[0])
		if _router.top_name() != step[1] or not _shows(step[2]):
			return _fail("клавиша не открыла экран %s:\n%s" % [step[1], _buffer()])
		var date: String = _router.view()["header"]["date"]
		for _day in STANDSTILL_DAYS:
			_router.tick(DAY_MILLISECONDS)
		if _router.view()["header"]["date"] != date:
			return _fail("на экране %s время шло: %s → %s" % [step[1], date, _router.view()["header"]["date"]])
		_press(KEY_ESCAPE)
		if _router.top_name() != "market":
			return _fail("Esc с экрана %s не вернул рынок:\n%s" % [step[1], _buffer()])
	return true


## F10 — меню выхода, «Сохранить», Enter на слоте 1; затем выход «F» в главное меню, «Загрузить»,
## Enter на слоте 1 — и снова рынок того же дня.
func _save_and_load() -> bool:
	var saved_date: String = _router.view()["header"]["date"]
	_press(KEY_F10)
	if _router.top_name() != "game_menu":
		return _fail("F10 не открыла меню выхода:\n" + _buffer())
	_press(DIGIT_KEYS[SAVE_ITEM])
	_press(KEY_ENTER)
	if not _shows("Жизнь записана в слот 1.") or not "1" in _saves.slots():
		return _fail("слот 1 не записан:\n" + _buffer())
	_press(KEY_ESCAPE)
	if not _to_main_menu():
		return false
	_press(DIGIT_KEYS[LOAD_ITEM])
	_press(KEY_ENTER)
	if _router.top_name() != "market" or _router.view()["header"]["date"] != saved_date:
		return _fail("слот 1 не вернул жизнь %s:\n%s" % [saved_date, _buffer()])
	return true


## Из жизни: меню выхода, последний пункт — «Выйти из игры», и окно «Abort, Retry, Fail?».
func _open_quit() -> bool:
	if _router.top_name() != "game_menu":
		_press(KEY_F10)
	var exits: Array = _router.view()["exits"]
	_press(DIGIT_KEYS[ITEMS_BEFORE_EXITS + exits.size() + ITEMS_AFTER_EXITS])
	if _router.top_name() != "quit" or not _shows("Abort, Retry, Fail?"):
		return _fail("последний пункт меню не открыл выход:\n" + _buffer())
	return true


## Ironman-жизнь сама ложится в слот; из главного меню «Продолжить» возвращает её, а новая
## ironman-жизнь поверх спрашивает подтверждения — Esc его отменяет.
func _ironman() -> bool:
	if not _to_main_menu() or not _new_life(false):
		return false
	var date: String = _router.view()["header"]["date"]
	if not _to_main_menu():
		return false
	if not Saves.IRONMAN_SLOT in _saves.slots():
		return _fail("ironman-жизнь не легла в свой слот: %s" % [_saves.slots()])
	_press(DIGIT_KEYS[CONTINUE_ITEM])
	var view := _router.view()
	if _router.top_name() != "market" or not view["ironman"] or view["header"]["date"] != date:
		return _fail("«Продолжить» не вернул ironman-жизнь %s:\n%s" % [date, _buffer()])
	if not _to_main_menu():
		return false
	_press(DIGIT_KEYS[NEW_LIFE_ITEM])
	_press(KEY_ENTER)
	if not _shows("Бросить незаконченную ironman-жизнь?"):
		return _fail("новая ironman-жизнь не спросила про прежнюю:\n" + _buffer())
	_press(KEY_ESCAPE)
	_press(KEY_ESCAPE)
	if _router.top_name() != "main_menu" or not Saves.IRONMAN_SLOT in _saves.slots():
		return _fail("отказ не вернул в меню с прежней ironman-жизнью:\n" + _buffer())
	return true


## Из жизни через «Abort, Retry, Fail?»: F — в главное меню.
func _to_main_menu() -> bool:
	if not _open_quit():
		return false
	_press(KEY_F)
	if _router.top_name() != "main_menu":
		return _fail("F не вернула в главное меню:\n" + _buffer())
	return true


## Из главного меню: F10 — выход, A — выйти в DOS.
func _quit() -> bool:
	_press(KEY_F10)
	if _router.top_name() != "quit" or not _shows("Abort, Retry, Fail?"):
		return _fail("F10 главного меню не открыла выход:\n" + _buffer())
	_press(KEY_A)
	if not _quit_asked:
		return _fail("A не попросила выйти из игры:\n" + _buffer())
	return true


func _settle_event() -> bool:
	var choices: Array = _router.view()["pending_event"]["choices"]
	var choice := choices.find_custom(func(shown: Dictionary) -> bool: return shown["available"]) + 1
	if choice < 1 or choice >= DIGIT_KEYS.size():
		return _fail("в событии нет выбора на цифру:\n" + _buffer())
	_press(DIGIT_KEYS[choice])
	if not _shows("Любая клавиша — продолжить"):
		return _fail("выбор %d не показал исход:\n%s" % [choice, _buffer()])
	_press(KEY_ENTER)
	return true


func _press(keycode: Key) -> void:
	var event := InputEventKey.new()
	event.keycode = keycode
	event.pressed = true
	_router.press(Keymap.action(event, _screen))


func _shows(text: String) -> bool:
	return text in _buffer()


func _buffer() -> String:
	var rows := PackedStringArray()
	for row in TextScreen.ROWS:
		rows.append(_screen.row_text(row))
	return "\n".join(rows)


func _clear_saves() -> void:
	var folder := SAVES_ROOT.path_join(Saves.SAVES_FOLDER)
	for slot in _saves.slots():
		DirAccess.remove_absolute(folder.path_join(slot + "." + Saves.EXTENSION))
	if _saves.has_past_lives():
		DirAccess.remove_absolute(SAVES_ROOT.path_join(Saves.META_FILE))
	if FileAccess.file_exists(SETTINGS_FILE):
		DirAccess.remove_absolute(SETTINGS_FILE)


func _finish(passed: bool) -> void:
	_screen.free()
	if passed:
		print("ui_flow: заставка с мелодией → меню → настройки → новая жизнь → рынок → бип отказа → покупка → пауза → событие → поездка → дела, связи, пейджер → сохранение → меню → загрузка → ironman → выход")
		quit()


func _fail(problem: String) -> bool:
	push_error("ui_flow: " + problem)
	quit(1)
	return false
