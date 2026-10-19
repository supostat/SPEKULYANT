class_name PagerScreen
extends RefCounted
## Пейджер, позже мобила (docs/SPEKULYANT.md §13.2, §13.3 п. 7, макет «Пейджер»): лента сообщений
## дел, крыши, рынка и близких — зелёным на чёрном, срочные красным. Экран открывается без
## выделения на последних сообщениях; ↑↓ выделяют и листают, Del стирает выделенное, Enter и F2 —
## перезвонить: пейджер только принимает, отвечают по телефону из связей.

const TEXT := 8
const BACKGROUND := 1
const WINDOW := Rect2i(12, 4, 56, 15)
const WINDOW_FOREGROUND := 10
const WINDOW_BACKGROUND := 0
const NO_DEVICE_TITLE := "ПЕЙДЖЕР"
const NO_DEVICE := "Пейджера при тебе нет: сообщения проходят мимо."
const EMPTY := "Сообщений нет."
const TIME_COLUMN := 15
const TIME_FOREGROUND := 2
const TEXT_COLUMN := 22
const TEXT_WIDTH := 44
const URGENT_FOREGROUND := 12
const FIRST_ROW := 6
const MESSAGE_ROWS := 2
const MESSAGES_SHOWN := 6
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 2
const HINT := "↑↓ листать   Enter ответить   Del стереть"
const HINT_ROW := 17
const WARNING_ROW := 21
const WARNING_COLUMN := 12
const WARNING_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Звонить"],
	["3", ""],
	["4", ""],
	["5", "Связи"],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Назад"],
]
const HELP_KEY := 1
const CALL_KEY := 2
const CONTACTS_KEY := 5
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const BACK_KEY := 10

var _messages := ListCursor.new(MESSAGES_SHOWN)
## Выделено ли сообщение: экран открывается без выделения, на последних сообщениях.
var _selecting := false
var _warning := ""


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	var pager: Variant = view["pager"]
	var title: String = NO_DEVICE_TITLE if pager == null else pager["device"]
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, WINDOW_FOREGROUND, WINDOW_BACKGROUND, title)
	if pager == null:
		_put_text(screen, FIRST_ROW, NO_DEVICE, WINDOW_FOREGROUND, WINDOW_BACKGROUND)
	else:
		_draw_messages(pager["messages"], screen)
		screen.put(TIME_COLUMN, HINT_ROW, HINT, TIME_FOREGROUND, WINDOW_BACKGROUND)
	screen.put(WARNING_COLUMN, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var count := _count(view)
	match action["action"]:
		"up", "down":
			if count == 0:
				return {}
			if not _selecting:
				_selecting = true
				_messages.select(count - 1, count)
			else:
				_messages.move(-1 if action["action"] == "up" else 1, count)
			return {"redraw": true}
		"delete":
			if _selecting and count > 0:
				return {"command": {"command": "erase_pager_message", "message": _messages.selected}}
		"confirm":
			return {"push": "contacts"}
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				CALL_KEY:
					return {"push": "contacts"}
				CONTACTS_KEY:
					return {"switch": "contacts"}
				BUSINESSES_KEY:
					return {"switch": "businesses"}
				PAGER_KEY, BACK_KEY:
					return {"pop": true}
	return {}


func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""


## Стёрто последнее — выделять больше нечего.
func follow(view: Dictionary) -> Dictionary:
	if _count(view) == 0:
		_selecting = false
	return {}


func _count(view: Dictionary) -> int:
	return 0 if view["pager"] == null else view["pager"]["messages"].size()


## Сообщение — две строки ленты: время и текст, перенос — на вторую строку.
func _draw_messages(messages: Array, screen: TextScreen) -> void:
	if messages.is_empty():
		_put_text(screen, FIRST_ROW, EMPTY, WINDOW_FOREGROUND, WINDOW_BACKGROUND)
		return
	if not _selecting:
		_messages.first = maxi(messages.size() - MESSAGES_SHOWN, 0)
	for index: int in _messages.shown(messages.size()):
		var message: Dictionary = messages[index]
		var row := FIRST_ROW + (index - _messages.first) * MESSAGE_ROWS
		var selected := _selecting and index == _messages.selected
		var background := SELECTED_BACKGROUND if selected else WINDOW_BACKGROUND
		var text_foreground := URGENT_FOREGROUND if message["urgent"] else WINDOW_FOREGROUND
		screen.put(TIME_COLUMN, row, message["time"], SELECTED_FOREGROUND if selected else TIME_FOREGROUND, background)
		var lines := TextScreen.wrap(message["text"], TEXT_WIDTH)
		for line_index in mini(lines.size(), MESSAGE_ROWS):
			var line: String = lines[line_index]
			if line_index == MESSAGE_ROWS - 1 and lines.size() > MESSAGE_ROWS:
				line = line.substr(0, TEXT_WIDTH - 1) + "…"
			screen.put(TEXT_COLUMN, row + line_index, line, SELECTED_FOREGROUND if selected else text_foreground, background)


func _put_text(screen: TextScreen, row: int, text: String, foreground: int, background: int) -> void:
	for line in TextScreen.wrap(text, WINDOW.size.x - 6):
		screen.put(TIME_COLUMN, row, line, foreground, background)
		row += 1
