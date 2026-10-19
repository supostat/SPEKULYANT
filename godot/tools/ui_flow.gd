extends SceneTree
## Безголовый клавиатурный сценарий для `cargo xtask ui-flow-smoke`: настоящее ядро через мост и стек
## экранов — рынок → F2 покупка в окне количества → пробел пауза и снова ход → событие → выбор →
## исход → F4 поездка → прибытие. Клавиши проходят через Keymap, как в игре; после каждого шага
## сверяется буфер экрана.

const LIFE := {"life": {"biography": "student", "ironman": false, "seed": 7}}
const DAY_MILLISECONDS := 1800
const LONGEST_WAIT_STEPS := 2000
const TRADE_GOOD := "chewing_gum"
const POCKET_LABEL := "Жвачка"
const DIGIT_KEYS := [KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9]

var _screen: TextScreen
var _router: Router


func _initialize() -> void:
	_screen = TextScreen.new()
	var game := SpeculantGame.new()
	var started: Dictionary = JSON.parse_string(game.new_life(JSON.stringify(LIFE)))
	if started.has("error"):
		_finish(_fail("жизнь не начинается: %s" % [started]))
		return
	_router = Router.new(game, _screen)
	_finish(_market() and _buy() and _pause() and _event() and _travel())


func _market() -> bool:
	if _router.top_name() != "market" or not _shows("Купить") or not _shows("Время идёт"):
		return _fail("рынок не открылся:\n" + _buffer())
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


func _finish(passed: bool) -> void:
	_screen.free()
	if passed:
		print("ui_flow: рынок → покупка → пауза → событие → исход → поездка → прибытие")
		quit()


func _fail(problem: String) -> bool:
	push_error("ui_flow: " + problem)
	quit(1)
	return false
