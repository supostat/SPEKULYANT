class_name MenuList
extends RefCounted
## Пункты меню с номерами: стрелки выбирают, Enter, цифра пункта или щелчок по его строке —
## выбирают его; закрытый пункт тусклый и не выбирается. Пункт — {"label", "enabled"}; какие они,
## решает экран по снимку.

const TEXT := 7
const BACKGROUND := 1
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const DISABLED_FOREGROUND := 8

var selected := 0
var _column: int
var _row: int
var _width: int


func _init(column: int, row: int, width: int) -> void:
	_column = column
	_row = row
	_width = width


func draw(items: Array, screen: TextScreen) -> void:
	selected = clampi(selected, 0, items.size() - 1)
	for index in items.size():
		var item: Dictionary = items[index]
		var line := TextScreen.pad_right(" %d. %s" % [index + 1, item["label"]], _width)
		var foreground := TEXT if item["enabled"] else DISABLED_FOREGROUND
		var background := BACKGROUND
		if index == selected:
			foreground = SELECTED_FOREGROUND
			background = SELECTED_BACKGROUND
		screen.put(_column, _row + index, line, foreground, background)


## {"chosen": номер пункта от нуля}, {"redraw": true} — выделение сдвинулось, или пусто.
func handle(action: Dictionary, items: Array) -> Dictionary:
	match action["action"]:
		"up":
			selected = clampi(selected - 1, 0, items.size() - 1)
			return {"redraw": true}
		"down":
			selected = clampi(selected + 1, 0, items.size() - 1)
			return {"redraw": true}
		"digit":
			return _choose(int(action["digit"]) - 1, items)
		"confirm":
			return _choose(selected, items)
		"click":
			var column: int = action["column"]
			if column >= _column and column < _column + _width:
				return _choose(int(action["row"]) - _row, items)
	return {}


func _choose(index: int, items: Array) -> Dictionary:
	if index < 0 or index >= items.size():
		return {}
	selected = index
	return {"chosen": index} if items[index]["enabled"] else {"redraw": true}
