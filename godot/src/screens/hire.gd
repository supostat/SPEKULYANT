class_name HireScreen
extends RefCounted
## Нанять управляющего (docs/SPEKULYANT.md §7.2, §7.3): люди, которых можно поставить на дело, —
## кто они, черты делениями и привычки; кому доверять, игрок решает сам. Enter — нанять; человек
## может и отказаться — отказ ядра остаётся на экране.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(1, 2, 78, 19)
const TITLE := "Нанять управляющего"
const BUSINESS_ROW := 3
const BUSINESS_FOREGROUND := 14
const LIST_COLUMN := 3
const LIST_ROW := 5
const LIST_WIDTH := 30
const LIST_VISIBLE := 12
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const DETAIL_COLUMN := 36
const DETAIL_WIDTH := 40
const NAME_FOREGROUND := 15
const TRAIT_LABEL_WIDTH := 11
const HABIT_FOREGROUND := 8
const NOBODY := "Сейчас нанять некого."
const MUTED_FOREGROUND := 8
const HINT := "↑↓ выбрать   Enter — нанять   Esc — назад"
const HINT_ROW := 19
const WARNING_ROW := 22
const WARNING_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Нанять"],
	["3", ""],
	["4", ""],
	["5", ""],
	["6", ""],
	["7", ""],
	["10", "Назад"],
]
const HELP_KEY := 1
const HIRE_KEY := 2
const BACK_KEY := 10

## Дело, на которое нанимают, — «тип@город»; пусто — первое из дел игрока.
var _business := ""
var _candidates := ListCursor.new(LIST_VISIBLE)
var _warning := ""
var _hired := false


## `{"business": "тип@город"}`.
func enter(arguments: Dictionary) -> void:
	_business = arguments["business"]


func draw(view: Dictionary, screen: TextScreen) -> void:
	var candidates: Array = view["candidates"]
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, TITLE)
	var business := _target(view)
	if not business.is_empty():
		screen.put(LIST_COLUMN, BUSINESS_ROW, business["title"], BUSINESS_FOREGROUND, BACKGROUND)
	if candidates.is_empty():
		screen.put(LIST_COLUMN, LIST_ROW, NOBODY, MUTED_FOREGROUND, BACKGROUND)
	for index: int in _candidates.shown(candidates.size()):
		var selected := index == _candidates.selected
		var line := TextScreen.pad_right(" " + candidates[index]["name"], LIST_WIDTH)
		screen.put(LIST_COLUMN, LIST_ROW + index - _candidates.first, line, SELECTED_FOREGROUND if selected else TEXT, SELECTED_BACKGROUND if selected else BACKGROUND)
	if not candidates.is_empty():
		_draw_candidate(candidates[_candidates.selected], screen)
	screen.put(LIST_COLUMN, HINT_ROW, HINT, MUTED_FOREGROUND, BACKGROUND)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var candidates: Array = view["candidates"]
	match action["action"]:
		"up":
			_candidates.move(-1, candidates.size())
			return {"redraw": true}
		"down":
			_candidates.move(1, candidates.size())
			return {"redraw": true}
		"click":
			var row := _candidates.row_at(action["row"], LIST_ROW, candidates.size())
			if row >= 0:
				_candidates.select(row, candidates.size())
				return {"redraw": true}
		"confirm":
			return _hire(view)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				HIRE_KEY:
					return _hire(view)
				BACK_KEY:
					return {"pop": true}
	return {}


func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""
	_hired = not answer.has("error")


## Нанятый уходит на дело — назад к экрану управляющего.
func follow(_view: Dictionary) -> Dictionary:
	if _hired:
		_hired = false
		return {"pop": true}
	return {}


func _hire(view: Dictionary) -> Dictionary:
	var candidates: Array = view["candidates"]
	var business := _target(view)
	if candidates.is_empty() or business.is_empty():
		return {}
	var candidate: String = candidates[_candidates.selected]["candidate"]
	return {"command": {"command": "hire", "business": business["business"], "candidate": candidate}}


func _target(view: Dictionary) -> Dictionary:
	return ManagerScreen.business_in(view, _business)


func _draw_candidate(candidate: Dictionary, screen: TextScreen) -> void:
	var row := LIST_ROW
	screen.put(DETAIL_COLUMN, row, candidate["name"], NAME_FOREGROUND, BACKGROUND)
	row += 1
	for line in TextScreen.wrap(candidate["role"], DETAIL_WIDTH):
		screen.put(DETAIL_COLUMN, row, line, TEXT, BACKGROUND)
		row += 1
	row += 1
	for level: Dictionary in candidate["traits"]:
		var bar := TextScreen.bar(int(level["level"]), int(level["scale"]))
		screen.put(DETAIL_COLUMN, row, TextScreen.pad_right(level["label"], TRAIT_LABEL_WIDTH) + bar, TEXT, BACKGROUND)
		row += 1
	row += 1
	for habit: String in candidate["habits"]:
		for line in TextScreen.wrap(habit, DETAIL_WIDTH):
			if row >= HINT_ROW - 1:
				return
			screen.put(DETAIL_COLUMN, row, line, HABIT_FOREGROUND, BACKGROUND)
			row += 1
