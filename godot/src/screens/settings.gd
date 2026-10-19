class_name SettingsScreen
extends RefCounted
## Настройки (docs/SPEKULYANT.md §13.1 п. 5–6): CRT-эффект и пищалка PC-speaker — включено или нет.
## ↑↓ выбирают, ◄ ►, Enter или щелчок переключают; перемена видна и слышна сразу и сохраняется.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const WINDOW := Rect2i(17, 6, 46, 12)
const TITLE := "Настройки"
const COLUMN := 20
const FIRST_ROW := 8
const ITEM_ROWS := 3
const LABEL_WIDTH := 22
const VALUE_WIDTH := 5
const VALUE_FOREGROUND := 15
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const NOTE_FOREGROUND := 8
const ON := "вкл"
const OFF := "выкл"
const ITEMS := [
	[Settings.CRT, "CRT-эффект", "строки развёртки и виньетка поверх экрана"],
	[Settings.SOUND, "Пищалка PC-speaker", "мелодия заставки и бипы событий"],
]
const HINT := "↑↓ выбрать   ◄► переключить   Esc назад"
const HINT_ROW := 15
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

var _selected := 0


func draw(view: Dictionary, screen: TextScreen) -> void:
	var settings: Dictionary = view["settings"]
	screen.clear(TEXT, BACKGROUND)
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, FRAME, BACKGROUND, TITLE)
	for index in ITEMS.size():
		var item: Array = ITEMS[index]
		var row := FIRST_ROW + index * ITEM_ROWS
		var label := TextScreen.dot_leader(item[1] + " ", LABEL_WIDTH)
		if index == _selected:
			screen.put(COLUMN, row, label, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
		else:
			screen.put(COLUMN, row, label, TEXT, BACKGROUND)
		var value := ON if settings[item[0]] else OFF
		screen.put(COLUMN + LABEL_WIDTH + 1, row, "◄ " + TextScreen.pad_right(value, VALUE_WIDTH) + " ►", VALUE_FOREGROUND, BACKGROUND)
		screen.put(COLUMN, row + 1, item[2], NOTE_FOREGROUND, BACKGROUND)
	screen.put(COLUMN, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, _view: Dictionary) -> Dictionary:
	match action["action"]:
		"up":
			_selected = maxi(_selected - 1, 0)
			return {"redraw": true}
		"down":
			_selected = mini(_selected + 1, ITEMS.size() - 1)
			return {"redraw": true}
		"left", "right", "confirm":
			return {"toggle": ITEMS[_selected][0]}
		"click":
			var offset: int = action["row"] - FIRST_ROW
			var index := floori(offset / float(ITEM_ROWS))
			if offset >= 0 and index < ITEMS.size():
				_selected = index
				return {"toggle": ITEMS[index][0]}
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
	return {}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}
