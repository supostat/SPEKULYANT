class_name NotebookScreen
extends RefCounted
## Записная книжка (docs/SPEKULYANT.md §11.3, §13.3 п. 8): знания, которые переходят из жизни в
## жизнь, — товары, люди и схемы. Открывается из главного меню и из меню выхода; листается.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(2, 1, 76, 22)
const TITLE := "Записная книжка"
const COLUMN := 4
const FIRST_ROW := 3
const VISIBLE := 16
const WIDTH := 72
const INDENT := "  "
const TOPIC_FOREGROUND := 14
const ENTRY_FOREGROUND := 15
const EMPTY := "Книжка пуста: её пишут прожитые жизни."
const EMPTY_FOREGROUND := 8
const HINT := "↑↓ PgUp PgDn — листать   Esc — назад"
const HINT_ROW := 21
const HINT_FOREGROUND := 8
const FKEYS := [
	["1", "Помощь"],
	["2", ""],
	["3", ""],
	["4", ""],
	["5", ""],
	["6", ""],
	["7", ""],
	["10", "Назад"],
]
const HELP_KEY := 1
const BACK_KEY := 10

var _text := ScrollText.new(VISIBLE)


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, TITLE)
	_text.draw(_lines(view["notebook"]), screen, COLUMN, FIRST_ROW, WIDTH, BACKGROUND)
	screen.put(COLUMN, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	match action["action"]:
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
			return {}
	return {"redraw": true} if _text.handle(action, _lines(view["notebook"]).size()) else {}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}


## Записи по темам: заголовок темы, название записи и её текст с отступом.
func _lines(notebook: Array) -> Array:
	if notebook.is_empty():
		return [[EMPTY, EMPTY_FOREGROUND]]
	var lines: Array = []
	var topic := ""
	for entry: Dictionary in notebook:
		if entry["topic"] != topic:
			if not lines.is_empty():
				lines.append(["", TEXT])
			topic = entry["topic"]
			lines.append([topic, TOPIC_FOREGROUND])
		lines.append([entry["title"], ENTRY_FOREGROUND])
		for line in TextScreen.wrap(entry["text"], WIDTH - INDENT.length()):
			lines.append([INDENT + line, TEXT])
	return lines
