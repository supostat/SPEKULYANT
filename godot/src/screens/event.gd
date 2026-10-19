class_name EventScreen
extends RefCounted
## Событие (docs/SPEKULYANT.md §13.3 п. 2, макет «Событие»): красное модальное окно с тенью поверх
## затемнённого рынка. Время стоит; выборы — цифрами 1–N или мышью по строке выбора; затем исход с
## переменами, и любая клавиша закрывает событие.

const DIM := 8
const BACKGROUND := 1
const WINDOW := Rect2i(13, 4, 54, 15)
const WINDOW_TITLE := "СОБЫТИЕ"
const WINDOW_FOREGROUND := 15
const WINDOW_BACKGROUND := 4
const TEXT_MARGIN := 3
const TEXT_TOP := 2
const TEXT_WIDTH := WINDOW.size.x - 2 * TEXT_MARGIN
const CHOICE_FOREGROUND := 14
const UNAVAILABLE_FOREGROUND := 7
const EFFECTS_FOREGROUND := 14
const HINT_FOREGROUND := 7
const STOPPED_HINT := "Время остановлено. Выбери %s."
const CONTINUE_HINT := "Любая клавиша — продолжить"
const GOODS_NAME_WIDTH := 44

## Строки выборов на экране — по ним мышь находит выбор.
var _choice_rows := {}


func draw(view: Dictionary, screen: TextScreen) -> void:
	_draw_backdrop(view, screen)
	var event: Dictionary = view["pending_event"]
	var left := WINDOW.position.x + TEXT_MARGIN
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, WINDOW_FOREGROUND, WINDOW_BACKGROUND, WINDOW_TITLE)
	var row := WINDOW.position.y + TEXT_TOP
	var hint_row := WINDOW.position.y + WINDOW.size.y - 2
	_choice_rows.clear()
	if event["outcome"] == null:
		row = _put_wrapped(screen, left, row, event["text"], WINDOW_FOREGROUND) + 1
		var choices: Array = event["choices"]
		for index in choices.size():
			var choice: Dictionary = choices[index]
			var line := "%d. %s" % [index + 1, choice["text"]]
			var color := CHOICE_FOREGROUND if choice["available"] else UNAVAILABLE_FOREGROUND
			screen.put(left, row, line, color, WINDOW_BACKGROUND)
			_choice_rows[row] = index + 1
			row += 1
		screen.put(left, hint_row, STOPPED_HINT % _numbers(choices.size()), HINT_FOREGROUND, WINDOW_BACKGROUND)
	else:
		var outcome: Dictionary = event["outcome"]
		row = _put_wrapped(screen, left, row, outcome["text"], WINDOW_FOREGROUND) + 1
		_put_wrapped(screen, left, row, outcome["effects"], EFFECTS_FOREGROUND)
		screen.put(left, hint_row, CONTINUE_HINT, HINT_FOREGROUND, WINDOW_BACKGROUND)


## Выбор цифрой или щелчком по его строке; после выбора любая клавиша читает исход.
func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var event: Dictionary = view["pending_event"]
	if event["outcome"] != null:
		return {"command": {"command": "continue"}}
	var choice := 0
	match action["action"]:
		"digit":
			choice = action["digit"]
		"click":
			choice = _choice_rows.get(action["row"], 0)
	var choices: Array = event["choices"]
	if choice < 1 or choice > choices.size() or not choices[choice - 1]["available"]:
		return {}
	return {"command": {"command": "choose", "choice": choice}}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}


## Под окном события — силуэт рынка: шапка, тусклые окна и названия товаров, строка F-клавиш.
func _draw_backdrop(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(MarketScreen.TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	var goods_box := MarketScreen.GOODS_BOX
	screen.box(goods_box.position.x, goods_box.position.y, goods_box.size.x, goods_box.size.y, DIM, BACKGROUND, view["market"]["place"])
	var goods: Array = view["market"]["goods"]
	for index in mini(goods.size(), MarketScreen.GOODS_VISIBLE):
		var name: String = goods[index]["name"]
		screen.put(MarketScreen.GOODS_COLUMN, MarketScreen.GOODS_FIRST_ROW + index, TextScreen.pad_right(name, GOODS_NAME_WIDTH), DIM, BACKGROUND)
	var pocket_box := MarketScreen.POCKET_BOX
	screen.box(pocket_box.position.x, pocket_box.position.y, pocket_box.size.x, pocket_box.size.y, DIM, BACKGROUND, MarketScreen.POCKET_TITLE)
	var messages_box := MarketScreen.MESSAGES_BOX
	screen.box(messages_box.position.x, messages_box.position.y, messages_box.size.x, messages_box.size.y, DIM, BACKGROUND, MarketScreen.MESSAGES_TITLE, false)
	screen.fkeys(MarketScreen.FKEYS)


## Текст по словам с первой строки `row`; возвращает строку после последней.
func _put_wrapped(screen: TextScreen, column: int, row: int, text: String, color: int) -> int:
	for line in TextScreen.wrap(text, TEXT_WIDTH):
		screen.put(column, row, line, color, WINDOW_BACKGROUND)
		row += 1
	return row


## «1–3» для подсказки, «1» — если выбор один.
static func _numbers(count: int) -> String:
	return "1" if count == 1 else "1–%d" % count
