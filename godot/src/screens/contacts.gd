class_name ContactsScreen
extends RefCounted
## Связи и соперники (docs/SPEKULYANT.md §8, §9, §13.3 п. 5): люди с именами — отношение, долг,
## на связи ли, история знакомства — и репутация у фракций шкалой от −предела до +предела. Enter
## и F2 звонят выбранному: человек на связи начинает событие о себе или не берёт трубку — это
## видно в строке под окнами. С соперниками говорят только в событиях.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const LIST_BOX := Rect2i(1, 2, 32, 19)
const LIST_TITLE := "Связи"
const LIST_COLUMN := 3
const LIST_ROW := 3
const LIST_VISIBLE := 17
const LIST_WIDTH := 28
const RIVALS_HEADING := "Соперники"
const HEADING_FOREGROUND := 14
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const UNREACHABLE_FOREGROUND := 8
const REACHABLE := "на связи"
const NOBODY := "Знакомых пока нет."
const DETAIL_BOX := Rect2i(34, 2, 45, 11)
const DETAIL_COLUMN := 36
const DETAIL_WIDTH := 41
const RELATION_LABEL := "Отношение: "
const AVAILABLE_FOREGROUND := 10
const DEBT_FOREGROUND := 14
const HISTORY_FOREGROUND := 7
const FACTIONS_BOX := Rect2i(34, 13, 45, 8)
const FACTIONS_TITLE := "Репутация"
const FACTION_WIDTH := 12
const CENTER := "│"
const NEGATIVE_FOREGROUND := 12
const POSITIVE_FOREGROUND := 10
const MINUS := "−"
const PLUS := "+"
const RIVAL_REFUSAL := "С соперником говорят только в событиях."
const MESSAGE_ROW := 22
const MESSAGE_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Звонить"],
	["3", ""],
	["4", ""],
	["5", ""],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Назад"],
]
const HELP_KEY := 1
const CALL_KEY := 2
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const BACK_KEY := 10

var _people := ListCursor.new(LIST_VISIBLE)
## Строка под окнами: отказ ядра, ответ на звонок или напоминание про соперников.
var _message := ""
var _called := false


func draw(view: Dictionary, screen: TextScreen) -> void:
	var people: Array = view["contacts"]
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_box(screen, LIST_BOX, LIST_TITLE, true)
	_draw_people(people, screen)
	if people.is_empty():
		_box(screen, DETAIL_BOX, "", false)
	else:
		_draw_person(people[_people.selected], screen)
	_draw_factions(view["factions"], screen)
	screen.put(1, MESSAGE_ROW, _message, MESSAGE_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var people: Array = view["contacts"]
	match action["action"]:
		"up":
			_people.move(-1, people.size())
		"down":
			_people.move(1, people.size())
		"click":
			var index := _person_at(action["row"], people)
			if index < 0:
				return {}
			_people.select(index, people.size())
		"confirm":
			return _call(people)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				CALL_KEY:
					return _call(people)
				BUSINESSES_KEY:
					return {"switch": "businesses"}
				PAGER_KEY:
					return {"switch": "pager"}
				BACK_KEY:
					return {"pop": true}
			return {}
		_:
			return {}
	_message = ""
	return {"redraw": true}


## Отказ ядра — не на связи, связь потеряна — остаётся строкой; удачный звонок покажет, что вышло.
func answered(answer: Dictionary) -> void:
	_called = not answer.has("error")
	_message = answer["message"] if answer.has("error") else ""


## После звонка без события в строке — последнее сообщение ленты: «… не берёт трубку».
func follow(view: Dictionary) -> Dictionary:
	if _called:
		_called = false
		var messages: Array = view["messages"]
		if not messages.is_empty():
			_message = messages.back()
	return {}


func _call(people: Array) -> Dictionary:
	if people.is_empty():
		return {}
	var person: Dictionary = people[_people.selected]
	if person["rival"]:
		_message = RIVAL_REFUSAL
		return {"redraw": true}
	return {"command": {"command": "call", "contact": person["contact"]}}


## Строки списка: знакомые, затем заголовок соперников и соперники; под заголовком строк нет.
func _rows(people: Array) -> Array:
	var rows: Array = []
	for index in people.size():
		if people[index]["rival"] and (index == 0 or not people[index - 1]["rival"]):
			rows.append(-1)
		rows.append(index)
	return rows


func _draw_people(people: Array, screen: TextScreen) -> void:
	if people.is_empty():
		screen.put(LIST_COLUMN, LIST_ROW, NOBODY, UNREACHABLE_FOREGROUND, BACKGROUND)
		return
	_people.select(_people.selected, people.size())
	var rows := _rows(people)
	var first := clampi(rows.find(_people.selected) - LIST_VISIBLE + 1, 0, maxi(rows.size() - LIST_VISIBLE, 0))
	for offset in mini(rows.size() - first, LIST_VISIBLE):
		var index: int = rows[first + offset]
		var row := LIST_ROW + offset
		if index < 0:
			screen.put(LIST_COLUMN, row, RIVALS_HEADING, HEADING_FOREGROUND, BACKGROUND)
			continue
		var person: Dictionary = people[index]
		var line := TextScreen.pad_right(" " + person["name"], LIST_WIDTH)
		if index == _people.selected:
			screen.put(LIST_COLUMN, row, line, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
		else:
			var reachable: bool = person["rival"] or person["availability"] == REACHABLE
			screen.put(LIST_COLUMN, row, line, TEXT if reachable else UNREACHABLE_FOREGROUND, BACKGROUND)


func _person_at(screen_row: int, people: Array) -> int:
	var rows := _rows(people)
	var first := clampi(rows.find(_people.selected) - LIST_VISIBLE + 1, 0, maxi(rows.size() - LIST_VISIBLE, 0))
	var offset := screen_row - LIST_ROW
	if offset < 0 or offset >= LIST_VISIBLE or first + offset >= rows.size():
		return -1
	return rows[first + offset]


func _draw_person(person: Dictionary, screen: TextScreen) -> void:
	_box(screen, DETAIL_BOX, person["name"], false)
	var lines: Array = []
	for line in TextScreen.wrap(person["role"], DETAIL_WIDTH):
		lines.append([line, TEXT])
	lines.append([RELATION_LABEL + person["relation"], TEXT])
	if person["debt"] != null:
		lines.append([person["debt"], DEBT_FOREGROUND])
	var reachable: bool = person["availability"] == REACHABLE
	lines.append([person["availability"], AVAILABLE_FOREGROUND if reachable else UNREACHABLE_FOREGROUND])
	var history: Array = person["history"]
	if not history.is_empty():
		lines.append(["", TEXT])
	var room := DETAIL_BOX.size.y - 2 - lines.size()
	var history_lines: Array = []
	for entry: String in history:
		for line in TextScreen.wrap(entry, DETAIL_WIDTH):
			history_lines.append([line, HISTORY_FOREGROUND])
	lines.append_array(history_lines.slice(maxi(history_lines.size() - room, 0)))
	for index in mini(lines.size(), DETAIL_BOX.size.y - 2):
		screen.put(DETAIL_COLUMN, DETAIL_BOX.position.y + 1 + index, lines[index][0], lines[index][1], BACKGROUND)


## Фракция — шкала из 2·предела делений с чертой посередине: закрашено от черты до уровня.
func _draw_factions(factions: Array, screen: TextScreen) -> void:
	_box(screen, FACTIONS_BOX, FACTIONS_TITLE, false)
	for index in mini(factions.size(), FACTIONS_BOX.size.y - 2):
		var faction: Dictionary = factions[index]
		var level := int(faction["level"])
		var limit := int(faction["limit"])
		var below := TextScreen.BAR_EMPTY.repeat(limit + mini(level, 0)) + TextScreen.BAR_FILLED.repeat(-mini(level, 0))
		var above := TextScreen.BAR_FILLED.repeat(maxi(level, 0)) + TextScreen.BAR_EMPTY.repeat(limit - maxi(level, 0))
		var sign := PLUS if level > 0 else (MINUS if level < 0 else "")
		var line: String = TextScreen.pad_right(faction["title"], FACTION_WIDTH) + below + CENTER + above + " " + sign + str(absi(level))
		var foreground := TEXT
		if level < 0:
			foreground = NEGATIVE_FOREGROUND
		elif level > 0:
			foreground = POSITIVE_FOREGROUND
		screen.put(DETAIL_COLUMN, FACTIONS_BOX.position.y + 1 + index, line, foreground, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, FRAME, BACKGROUND, title, is_double)
