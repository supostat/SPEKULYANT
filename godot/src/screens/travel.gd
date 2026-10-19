class_name TravelScreen
extends RefCounted
## Поездка (docs/SPEKULYANT.md §13.3 п. 3, правила §XIII дизайн-книги): направления из города со
## сроком и ценой проезда, «по блату» — через знакомого; Enter — в путь. В пути окно показывает,
## откуда и куда, день прибытия и сколько осталось; Enter перематывает до прибытия, а события
## дороги приходят своим окном. По прибытии экран уступает место рынку.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const WINDOW := Rect2i(2, 2, 74, 19)
const COLUMN := 5
const HEADINGS := ["Куда", "Дней", "Проезд", "Как"]
const HEADINGS_FOREGROUND := 14
const FIRST_ROW := 5
const DESTINATIONS_VISIBLE := 12
const NAME_WIDTH := 22
const DAYS_WIDTH := 6
const FARE_WIDTH := 13
const ACCESS_WIDTH := 22
const LINE_WIDTH := NAME_WIDTH + DAYS_WIDTH + FARE_WIDTH + 3 + ACCESS_WIDTH
const BY_CONNECTIONS := "по блату"
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const UNAVAILABLE_FOREGROUND := 8
const WARNING_FOREGROUND := 14
const HINT_FOREGROUND := 8
const CHOOSING_HINT := "↑↓ выбрать   Enter ехать   Esc назад"
const JOURNEY_HINT := "Enter — перемотать до прибытия   пробел — пауза"
const ROUTE_FOREGROUND := 15
const CHOOSING_TITLE := "Поездка"
const JOURNEY_TITLE := "В пути"
const ARRIVAL_LABEL := "Прибытие: "
const DAYS_LEFT_LABEL := "Дней в пути осталось: "
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
const SLOWEST := 1
const FASTEST := 3

var _destinations := ListCursor.new(DESTINATIONS_VISIBLE)
var _was_traveling := false
var _warning := ""


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	var travel: Dictionary = view["travel"]
	var journey: Variant = travel["journey"]
	var title := JOURNEY_TITLE if journey != null else CHOOSING_TITLE
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, FRAME, BACKGROUND, title)
	if journey != null:
		_draw_journey(journey, screen)
	else:
		_draw_destinations(travel["destinations"], screen)
	var warning_row := WINDOW.position.y + WINDOW.size.y - 3
	screen.put(COLUMN, warning_row, _warning, WARNING_FOREGROUND, BACKGROUND)
	var hint := JOURNEY_HINT if journey != null else CHOOSING_HINT
	screen.put(COLUMN, WINDOW.position.y + WINDOW.size.y - 2, hint, HINT_FOREGROUND, BACKGROUND)
	TimeIndicator.draw(view["time"], screen)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	match action["action"]:
		"pause":
			return {"command": {"command": "pause", "paused": not view["time"]["paused"]}}
		"faster":
			return {"command": {"command": "set_speed", "speed": mini(int(view["time"]["speed"]) + 1, FASTEST)}}
		"slower":
			return {"command": {"command": "set_speed", "speed": maxi(int(view["time"]["speed"]) - 1, SLOWEST)}}
	if view["travel"]["journey"] != null:
		if action["action"] == "confirm":
			return {"command": {"command": "fast_forward", "until": "until_arrival"}}
		return {}
	var destinations: Array = view["travel"]["destinations"]
	match action["action"]:
		"up":
			_destinations.move(-1, destinations.size())
			return {"redraw": true}
		"down":
			_destinations.move(1, destinations.size())
			return {"redraw": true}
		"click":
			var row := _destinations.row_at(action["row"], FIRST_ROW, destinations.size())
			if row >= 0:
				_destinations.select(row, destinations.size())
				return {"redraw": true}
		"confirm":
			if destinations.is_empty() or not destinations[_destinations.selected]["available"]:
				return {}
			return {"command": {"command": "travel", "destination": destinations[_destinations.selected]["city"]}}
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
	return {}


## Отказ ядра остаётся в окне до следующего ответа.
func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""


## Поездка кончилась — экран уступает место рынку нового города.
func follow(view: Dictionary) -> Dictionary:
	var traveling: bool = view["travel"]["journey"] != null
	var arrived := _was_traveling and not traveling
	_was_traveling = traveling
	return {"pop": true} if arrived else {}


func _draw_destinations(destinations: Array, screen: TextScreen) -> void:
	var headings: String = (
		TextScreen.pad_right(HEADINGS[0], NAME_WIDTH)
		+ TextScreen.pad_left(HEADINGS[1], DAYS_WIDTH)
		+ TextScreen.pad_left(HEADINGS[2], FARE_WIDTH)
		+ "   "
		+ HEADINGS[3]
	)
	screen.put(COLUMN, WINDOW.position.y + 1, headings, HEADINGS_FOREGROUND, BACKGROUND)
	for index: int in _destinations.shown(destinations.size()):
		var destination: Dictionary = destinations[index]
		var line: String = (
			TextScreen.pad_right(destination["name"], NAME_WIDTH)
			+ TextScreen.pad_left(str(int(destination["days"])), DAYS_WIDTH)
			+ TextScreen.pad_left(destination["fare"], FARE_WIDTH)
			+ "   "
			+ (BY_CONNECTIONS if destination["by_connections"] else "")
		)
		var foreground := TEXT if destination["available"] else UNAVAILABLE_FOREGROUND
		var background := BACKGROUND
		if index == _destinations.selected:
			foreground = SELECTED_FOREGROUND
			background = SELECTED_BACKGROUND
		screen.put(COLUMN, FIRST_ROW + index - _destinations.first, TextScreen.pad_right(line, LINE_WIDTH), foreground, background)


func _draw_journey(journey: Dictionary, screen: TextScreen) -> void:
	var top := WINDOW.position.y
	screen.put(COLUMN, top + 2, "%s → %s" % [journey["from"], journey["to"]], ROUTE_FOREGROUND, BACKGROUND)
	screen.put(COLUMN, top + 4, ARRIVAL_LABEL + journey["arrives"], TEXT, BACKGROUND)
	screen.put(COLUMN, top + 5, DAYS_LEFT_LABEL + str(int(journey["days_left"])), TEXT, BACKGROUND)
