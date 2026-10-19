class_name BusinessesScreen
extends RefCounted
## Дела (docs/SPEKULYANT.md §7, §13.3 п. 4): свои дела — город, форма, управляющий — и дела,
## которые можно открыть в городе сейчас, с ценой точки. Дело без управляющего — жёлтым, закрытое —
## красным. Стрелки идут по обоим спискам; Enter на своём деле — экран управляющего, на новом — F2
## или Enter открывают точку.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const COLUMN := 3
const LINE_WIDTH := 74
const HEADING_FOREGROUND := 14
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const OWNED_BOX := Rect2i(1, 2, 78, 11)
const OWNED_TITLE := "Дела"
const OWNED_HEADINGS := ["Дело", "Город", "Форма", "Управляющий"]
const OWNED_ROW := 4
const OWNED_VISIBLE := 7
const TITLE_WIDTH := 34
const CITY_WIDTH := 14
const FORM_WIDTH := 14
const NO_MANAGER := "нет"
const IDLE_FOREGROUND := 14
const CLOSED_FOREGROUND := 12
const NOTHING_OWNED := "Своих дел пока нет."
const OWNED_HINT := "↑↓ выбрать   Enter — управляющий"
const KINDS_BOX := Rect2i(1, 13, 78, 8)
const KINDS_TITLE := "Открыть дело"
const KINDS_HEADINGS := ["Дело", "Форма", "Точка"]
const KINDS_ROW := 15
const KINDS_VISIBLE := 4
const KIND_FORM_WIDTH := 20
const COST_WIDTH := 14
const UNAVAILABLE_FOREGROUND := 8
const NOTHING_TO_OPEN := "Здесь и сейчас открыть нечего."
const KINDS_HINT := "Enter или F2 — открыть точку за наличные"
const MUTED_FOREGROUND := 8
const WARNING_ROW := 22
const WARNING_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Открыть"],
	["3", ""],
	["4", "Деньги"],
	["5", "Связи"],
	["6", ""],
	["7", "Пейджер"],
	["10", "Назад"],
]
const HELP_KEY := 1
const OPEN_KEY := 2
const MONEY_KEY := 4
const CONTACTS_KEY := 5
const PAGER_KEY := 7
const BACK_KEY := 10

var _owned := ListCursor.new(OWNED_VISIBLE)
var _kinds := ListCursor.new(KINDS_VISIBLE)
## Выделение в списке новых дел, а не своих.
var _on_kinds := false
var _placed := false
var _warning := ""


func draw(view: Dictionary, screen: TextScreen) -> void:
	var owned: Array = view["businesses"]
	var kinds: Array = view["business_kinds"]
	if not _placed:
		_on_kinds = owned.is_empty()
		_placed = true
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_box(screen, OWNED_BOX, OWNED_TITLE, true)
	var owned_headings: String = TextScreen.pad_right(OWNED_HEADINGS[0], TITLE_WIDTH) + TextScreen.pad_right(OWNED_HEADINGS[1], CITY_WIDTH) + TextScreen.pad_right(OWNED_HEADINGS[2], FORM_WIDTH) + OWNED_HEADINGS[3]
	screen.put(COLUMN, OWNED_BOX.position.y + 1, owned_headings, HEADING_FOREGROUND, BACKGROUND)
	if owned.is_empty():
		screen.put(COLUMN, OWNED_ROW, NOTHING_OWNED, MUTED_FOREGROUND, BACKGROUND)
	for index: int in _owned.shown(owned.size()):
		var business: Dictionary = owned[index]
		var manager: String = NO_MANAGER if business["manager"] == null else business["manager"]["name"]
		var line: String = TextScreen.pad_right(business["title"], TITLE_WIDTH) + TextScreen.pad_right(business["city"], CITY_WIDTH) + TextScreen.pad_right(business["form"], FORM_WIDTH) + manager
		var foreground := TEXT
		if business["closed_until"] != null:
			foreground = CLOSED_FOREGROUND
		elif business["manager"] == null:
			foreground = IDLE_FOREGROUND
		_put_row(screen, OWNED_ROW + index - _owned.first, line, foreground, not _on_kinds and index == _owned.selected)
	screen.put(COLUMN, OWNED_BOX.end.y - 2, OWNED_HINT, MUTED_FOREGROUND, BACKGROUND)
	_box(screen, KINDS_BOX, KINDS_TITLE, false)
	var kind_headings: String = TextScreen.pad_right(KINDS_HEADINGS[0], TITLE_WIDTH) + TextScreen.pad_right(KINDS_HEADINGS[1], KIND_FORM_WIDTH) + TextScreen.pad_left(KINDS_HEADINGS[2], COST_WIDTH)
	screen.put(COLUMN, KINDS_BOX.position.y + 1, kind_headings, HEADING_FOREGROUND, BACKGROUND)
	if kinds.is_empty():
		screen.put(COLUMN, KINDS_ROW, NOTHING_TO_OPEN, MUTED_FOREGROUND, BACKGROUND)
	for index: int in _kinds.shown(kinds.size()):
		var kind: Dictionary = kinds[index]
		var line: String = TextScreen.pad_right(kind["title"], TITLE_WIDTH) + TextScreen.pad_right(kind["form"], KIND_FORM_WIDTH) + TextScreen.pad_left(kind["cost"], COST_WIDTH)
		var foreground := TEXT if kind["available"] else UNAVAILABLE_FOREGROUND
		_put_row(screen, KINDS_ROW + index - _kinds.first, line, foreground, _on_kinds and index == _kinds.selected)
	screen.put(COLUMN, KINDS_BOX.end.y - 2, KINDS_HINT, MUTED_FOREGROUND, BACKGROUND)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var owned: Array = view["businesses"]
	var kinds: Array = view["business_kinds"]
	match action["action"]:
		"up":
			if _on_kinds and _kinds.selected == 0 and not owned.is_empty():
				_on_kinds = false
				_owned.select(owned.size() - 1, owned.size())
			elif _on_kinds:
				_kinds.move(-1, kinds.size())
			else:
				_owned.move(-1, owned.size())
			return {"redraw": true}
		"down":
			if not _on_kinds and _owned.selected >= owned.size() - 1 and not kinds.is_empty():
				_on_kinds = true
				_kinds.select(0, kinds.size())
			elif _on_kinds:
				_kinds.move(1, kinds.size())
			else:
				_owned.move(1, owned.size())
			return {"redraw": true}
		"click":
			var owned_row := _owned.row_at(action["row"], OWNED_ROW, owned.size())
			var kind_row := _kinds.row_at(action["row"], KINDS_ROW, kinds.size())
			if owned_row >= 0:
				_on_kinds = false
				_owned.select(owned_row, owned.size())
				return {"redraw": true}
			if kind_row >= 0:
				_on_kinds = true
				_kinds.select(kind_row, kinds.size())
				return {"redraw": true}
		"confirm":
			if _on_kinds:
				return _open(kinds)
			if not owned.is_empty():
				return {"push": "manager", "with": {"business": owned[_owned.selected]["business"]}}
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				OPEN_KEY:
					return _open(kinds) if _on_kinds else {}
				MONEY_KEY:
					return {"switch": "money"}
				CONTACTS_KEY:
					return {"switch": "contacts"}
				PAGER_KEY:
					return {"switch": "pager"}
				BACK_KEY:
					return {"pop": true}
	return {}


## Отказ ядра — не хватает денег, предел точек — под окнами до следующего ответа.
func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _open(kinds: Array) -> Dictionary:
	if kinds.is_empty():
		return {}
	return {"command": {"command": "open_business", "kind": kinds[_kinds.selected]["kind"]}}


func _put_row(screen: TextScreen, row: int, line: String, foreground: int, selected: bool) -> void:
	var text := TextScreen.pad_right(line, LINE_WIDTH)
	if selected:
		screen.put(COLUMN, row, text, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
	else:
		screen.put(COLUMN, row, text, foreground, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, FRAME, BACKGROUND, title, is_double)
