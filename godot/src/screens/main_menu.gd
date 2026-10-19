class_name MainMenuScreen
extends RefCounted
## Главное меню лобби (§11.4, §13.3 п. 11): продолжить ironman-жизнь из её слота, начать новую,
## загрузить жизнь свободного режима, открыть записную книжку и рекорды или выйти в DOS. Пункты,
## которым нечего открыть, тусклые.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const TITLE := " СПЕКУЛЯНТЪ "
const TITLE_FOREGROUND := 15
const TITLE_BACKGROUND := 4
const VERSION_LINE := " версия 1.0 "
const VERSION_FOREGROUND := 0
const VERSION_BACKGROUND := 3
const MENU_BOX := Rect2i(24, 6, 32, 12)
const MENU_TITLE := "Главное меню"
const MENU_COLUMN := 26
const MENU_ROW := 8
const MENU_WIDTH := 28
const HINT := "↑↓, цифра, Enter"
const HINT_ROW := 15
const HINT_FOREGROUND := 8
const WARNING_ROW := 20
const WARNING_FOREGROUND := 14
const WARNING_COLUMN := 2
const WARNING_WIDTH := 76
const FOOTER := "(с) 1993 ТОО «Перестройка-Софт»"
const FOOTER_ROW := 22
const FOOTER_FOREGROUND := 8
const FKEYS := [
	["1", "Помощь"],
	["2", ""],
	["3", ""],
	["4", ""],
	["5", ""],
	["6", ""],
	["7", ""],
	["10", "Выход"],
]
const HELP_KEY := 1
const EXIT_KEY := 10
const ITEMS := [
	["continue", "Продолжить жизнь"],
	["new_life", "Новая жизнь"],
	["load", "Загрузить"],
	["notebook", "Записная книжка"],
	["records", "Рекорды"],
	["quit", "Выход в DOS"],
]

var _menu := MenuList.new(MENU_COLUMN, MENU_ROW, MENU_WIDTH)
var _warning := ""
var _placed := false


func draw(view: Dictionary, screen: TextScreen) -> void:
	var items := _items(view)
	if not _placed:
		_menu.selected = 0 if items[0]["enabled"] else 1
		_placed = true
	screen.clear(TEXT, BACKGROUND)
	screen.put(0, 0, TITLE, TITLE_FOREGROUND, TITLE_BACKGROUND)
	screen.put(TITLE.length(), 0, TextScreen.pad_right(VERSION_LINE, TextScreen.COLUMNS - TITLE.length()), VERSION_FOREGROUND, VERSION_BACKGROUND)
	screen.shadow(MENU_BOX.position.x, MENU_BOX.position.y, MENU_BOX.size.x, MENU_BOX.size.y)
	screen.box(MENU_BOX.position.x, MENU_BOX.position.y, MENU_BOX.size.x, MENU_BOX.size.y, FRAME, BACKGROUND, MENU_TITLE)
	_menu.draw(items, screen)
	screen.put(MENU_COLUMN + 1, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	var warning := TextScreen.wrap(_warning, WARNING_WIDTH)
	for index in warning.size():
		screen.put(WARNING_COLUMN, WARNING_ROW + index, warning[index], WARNING_FOREGROUND, BACKGROUND)
	screen.put(floori((TextScreen.COLUMNS - FOOTER.length()) / 2.0), FOOTER_ROW, FOOTER, FOOTER_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if action["action"] == "fkey":
		match action["key"]:
			HELP_KEY:
				return {"push": "help"}
			EXIT_KEY:
				return {"push": "quit"}
		return {}
	if action["action"] == "back":
		return {"push": "quit"}
	var picked := _menu.handle(action, _items(view))
	if not picked.has("chosen"):
		return picked
	_warning = ""
	match ITEMS[picked["chosen"]][0]:
		"continue":
			return {"resume": Saves.IRONMAN_SLOT}
		"new_life":
			return {"push": "new_life"}
		"load":
			return {"push": "slots", "with": {"mode": SlotsScreen.LOAD}}
		"notebook":
			return {"push": "notebook"}
		"records":
			return {"push": "records"}
	return {"push": "quit"}


## Отказ ядра продолжить или загрузить жизнь остаётся под меню.
func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _items(view: Dictionary) -> Array:
	var slots: Array = view["slots"]
	var free := slots.any(func(saved: Dictionary) -> bool: return saved["slot"] != Saves.IRONMAN_SLOT)
	var enabled := {"continue": Saves.holds_ironman(view), "load": free}
	var items: Array = []
	for item: Array in ITEMS:
		items.append({"label": item[1], "enabled": enabled.get(item[0], true)})
	return items
