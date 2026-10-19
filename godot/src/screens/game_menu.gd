class_name GameMenuScreen
extends RefCounted
## Меню выхода в жизни (F10 или Esc на рынке и в зоне; docs/SPEKULYANT.md §11.2, §11.4): вернуться,
## сохранить и загрузить — только в свободном режиме, ironman сохраняется сам, — записная книжка,
## добровольный уход — эмиграция или покой, с подтверждением, — и выход из игры.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const WINDOW := Rect2i(22, 4, 36, 15)
const TITLE := "Выход"
const MENU_COLUMN := 24
const MENU_ROW := 6
const MENU_WIDTH := 32
const IRONMAN_NOTE := "Ironman сохраняется сам."
const NOTE_FOREGROUND := 8
const RETIRE_QUESTION := "%s?"
const RETIRE_DETAIL := "Жизнь кончится итогами: капитал, сделки, открытия."
const RETIRE_VERB := "уйти"
const WARNING_ROW := 21
const WARNING_FOREGROUND := 14
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

var _menu := MenuList.new(MENU_COLUMN, MENU_ROW, MENU_WIDTH)
var _retire: ConfirmDialog
var _ending := ""
var _warning := ""


func draw(view: Dictionary, screen: TextScreen) -> void:
	var items := _items(view)
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, FRAME, BACKGROUND, TITLE)
	_menu.draw(items, screen)
	if view["ironman"]:
		screen.put(MENU_COLUMN + 1, WINDOW.end.y - 2, IRONMAN_NOTE, NOTE_FOREGROUND, BACKGROUND)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)
	if _retire != null:
		_retire.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if _retire != null:
		return _handle_retire(action)
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
	var items := _items(view)
	var picked := _menu.handle(action, items)
	if not picked.has("chosen"):
		return picked
	var item: Dictionary = items[picked["chosen"]]
	match item["id"]:
		"back":
			return {"pop": true}
		"save":
			return {"push": "slots", "with": {"mode": SlotsScreen.SAVE}}
		"load":
			return {"push": "slots", "with": {"mode": SlotsScreen.LOAD}}
		"notebook":
			return {"push": "notebook"}
		"retire":
			_ending = item["ending"]
			_retire = ConfirmDialog.new(TITLE, RETIRE_QUESTION % item["label"], RETIRE_DETAIL, RETIRE_VERB)
			return {"redraw": true}
	return {"push": "quit"}


## Отказ ядра уйти остаётся в окне подтверждения; удачный уход ядро само ставит на экран итогов.
func answered(answer: Dictionary) -> void:
	if _retire != null and answer.has("error"):
		_retire.warning = answer["message"]
	else:
		_retire = null
		_warning = answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _items(view: Dictionary) -> Array:
	var free: bool = not view["ironman"]
	var items: Array = [
		{"id": "back", "label": "Вернуться к жизни", "enabled": true},
		{"id": "save", "label": "Сохранить", "enabled": free},
		{"id": "load", "label": "Загрузить", "enabled": free},
		{"id": "notebook", "label": "Записная книжка", "enabled": true},
	]
	for exit: Dictionary in view["exits"]:
		items.append({"id": "retire", "ending": exit["ending"], "label": exit["title"], "enabled": exit["available"]})
	items.append({"id": "quit", "label": "Выйти из игры", "enabled": true})
	return items


func _handle_retire(action: Dictionary) -> Dictionary:
	match _retire.handle(action):
		ConfirmDialog.CONFIRMED:
			return {"command": {"command": "retire", "ending": _ending}}
		ConfirmDialog.CANCELLED:
			_retire = null
			return {"redraw": true}
	return {}
