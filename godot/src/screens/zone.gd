class_name ZoneScreen
extends RefCounted
## Зона (docs/SPEKULYANT.md §12, §13.3 п. 9): экран режима, на который ядро ставит жизнь после
## приговора. Срок и день выхода, авторитет, сигареты и чай, промзона, передача, УДО и взятка, если
## зона эпохи её берёт, услуги за пачки и доверенные лица на воле; неделя идёт за время дня на
## воле. F2 — ходить на промзону или нет, F3 — передача, F4 — УДО, F5 — взятка, ↑↓ и Enter — услуга;
## дела и пейджер — те же экраны, что на воле.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const MUTED_FOREGROUND := 8
const CAMP_BOX := Rect2i(1, 2, 40, 15)
const COLUMN := 3
const LABEL_WIDTH := 16
const VALUE_COLUMN := 20
const SENTENCE_LABEL := "Срок"
const RELEASE_LABEL := "Выход по сроку"
const AUTHORITY_LABEL := "Авторитет"
const AUTHORITY_FOREGROUND := 11
const WORK_LABEL := "Промзона"
const WORKING := "ходишь"
const IDLE := "не ходишь"
const WORKING_FOREGROUND := 10
const PARCEL_LABEL := "Передача"
const PAROLE_LABEL := "УДО"
const BRIBE_LABEL := "Взятка"
const NO_BRIBE := "здесь не берут"
const OFFER_FOREGROUND := 15
const SERVICES_BOX := Rect2i(42, 2, 37, 8)
const SERVICES_TITLE := "Услуги"
const SERVICE_COLUMN := 44
const SERVICE_ROW := 3
## Услуга — две строки: название и цена в пачках под ним.
const SERVICE_ROWS := 2
const SERVICES_VISIBLE := 2
const SERVICE_WIDTH := 33
const PRICE_INDENT := "  "
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const SERVICES_HINT := "↑↓ выбрать   Enter — заплатить"
const TRUSTEES_BOX := Rect2i(42, 10, 37, 7)
const TRUSTEES_TITLE := "На воле"
const NO_TRUSTEES := "Дел на воле нет."
const MESSAGES_BOX := Rect2i(1, 17, 78, 5)
const MESSAGES_TITLE := "Сообщения"
const MESSAGES_SHOWN := 3
const MESSAGE_WIDTH := 74
const MESSAGE_PREFIX := "> "
const WARNING_ROW := 22
const WARNING_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Промзона"],
	["3", "Передача"],
	["4", "УДО"],
	["5", "Взятка"],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Выход"],
]
const HELP_KEY := 1
const WORK_KEY := 2
const PARCEL_KEY := 3
const PAROLE_KEY := 4
const BRIBE_KEY := 5
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const EXIT_KEY := 10
const SLOWEST := 1
const FASTEST := 3

var _services := ListCursor.new(SERVICES_VISIBLE)
var _warning := ""


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	var zone: Dictionary = view["zone"]
	_draw_camp(zone, screen)
	_draw_services(zone["services"], screen)
	_draw_trustees(zone["trustees"], screen)
	_draw_messages(view["messages"], screen)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	TimeIndicator.draw(view["time"], screen)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var zone: Dictionary = view["zone"]
	var services: Array = zone["services"]
	match action["action"]:
		"up":
			_services.move(-1, services.size())
			return {"redraw": true}
		"down":
			_services.move(1, services.size())
			return {"redraw": true}
		"click":
			var offset: int = action["row"] - SERVICE_ROW
			var clicked := _services.first + floori(offset / float(SERVICE_ROWS))
			if action["column"] >= SERVICE_COLUMN and offset >= 0 and offset < SERVICES_VISIBLE * SERVICE_ROWS and clicked < services.size():
				_services.select(clicked, services.size())
				return {"redraw": true}
		"confirm":
			if not services.is_empty():
				return {"command": {"command": "use_service", "service": services[_services.selected]["service"]}}
		"back":
			return {"push": "game_menu"}
		"pause":
			return {"command": {"command": "pause", "paused": not view["time"]["paused"]}}
		"faster":
			return {"command": {"command": "set_speed", "speed": mini(int(view["time"]["speed"]) + 1, FASTEST)}}
		"slower":
			return {"command": {"command": "set_speed", "speed": maxi(int(view["time"]["speed"]) - 1, SLOWEST)}}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				WORK_KEY:
					return {"command": {"command": "set_work", "work": not zone["working"]}}
				PARCEL_KEY:
					return {"command": {"command": "order_parcel"}}
				PAROLE_KEY:
					return {"command": {"command": "apply_for_parole"}}
				BRIBE_KEY:
					return {"command": {"command": "bribe_for_release"}}
				BUSINESSES_KEY:
					return {"push": "businesses"}
				PAGER_KEY:
					return {"push": "pager"}
				EXIT_KEY:
					return {"push": "game_menu"}
	return {}


## Отказ ядра — рано на УДО, не хватает пачек — строкой под окнами до следующего ответа.
func answered(answer: Dictionary) -> void:
	_warning = answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _draw_camp(zone: Dictionary, screen: TextScreen) -> void:
	_box(screen, CAMP_BOX, zone["camp"], true)
	var row := CAMP_BOX.position.y + 2
	_line(screen, row, SENTENCE_LABEL, zone["sentence"], TEXT)
	_line(screen, row + 1, RELEASE_LABEL, zone["release"], TEXT)
	var authority: Dictionary = zone["authority"]
	var bar := TextScreen.bar(int(authority["level"]), int(authority["scale"]))
	_line(screen, row + 3, AUTHORITY_LABEL, "%s %s" % [bar, authority["label"]], AUTHORITY_FOREGROUND)
	row += 4
	for item: Dictionary in zone["stash"]:
		_line(screen, row, item["label"], item["amount"], TEXT)
		row += 1
	var working: bool = zone["working"]
	_line(screen, row + 1, WORK_LABEL, WORKING if working else IDLE, WORKING_FOREGROUND if working else MUTED_FOREGROUND)
	_offer(screen, row + 2, PARCEL_LABEL, zone["parcel"])
	_offer(screen, row + 3, PAROLE_LABEL, zone["parole"])
	if zone["bribe"] != null:
		_offer(screen, row + 4, BRIBE_LABEL, zone["bribe"])
	else:
		_line(screen, row + 4, BRIBE_LABEL, NO_BRIBE, MUTED_FOREGROUND)


func _line(screen: TextScreen, row: int, label: String, value: String, foreground: int) -> void:
	screen.put(COLUMN, row, TextScreen.dot_leader(label, LABEL_WIDTH), TEXT, BACKGROUND)
	screen.put(VALUE_COLUMN, row, value, foreground, BACKGROUND)


## Передача, УДО, взятка: цена или день, с которого можно, — ярко, когда можно сейчас.
func _offer(screen: TextScreen, row: int, label: String, offer: Dictionary) -> void:
	_line(screen, row, label, offer["terms"], OFFER_FOREGROUND if offer["available"] else MUTED_FOREGROUND)


func _draw_services(services: Array, screen: TextScreen) -> void:
	_box(screen, SERVICES_BOX, SERVICES_TITLE, false)
	for index: int in _services.shown(services.size()):
		var service: Dictionary = services[index]
		var row := SERVICE_ROW + (index - _services.first) * SERVICE_ROWS
		var foreground := TEXT if service["available"] else MUTED_FOREGROUND
		var title := TextScreen.pad_right(service["title"], SERVICE_WIDTH)
		if index == _services.selected:
			screen.put(SERVICE_COLUMN, row, title, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
		else:
			screen.put(SERVICE_COLUMN, row, title, foreground, BACKGROUND)
		screen.put(SERVICE_COLUMN, row + 1, TextScreen.pad_right(PRICE_INDENT + service["price"], SERVICE_WIDTH), foreground, BACKGROUND)
	screen.put(SERVICE_COLUMN, SERVICES_BOX.end.y - 2, SERVICES_HINT, MUTED_FOREGROUND, BACKGROUND)


func _draw_trustees(trustees: Array, screen: TextScreen) -> void:
	_box(screen, TRUSTEES_BOX, TRUSTEES_TITLE, false)
	var lines := PackedStringArray()
	for trustee: String in trustees:
		lines.append_array(TextScreen.wrap(trustee, SERVICE_WIDTH))
	if lines.is_empty():
		screen.put(SERVICE_COLUMN, TRUSTEES_BOX.position.y + 1, NO_TRUSTEES, MUTED_FOREGROUND, BACKGROUND)
	for index in mini(lines.size(), TRUSTEES_BOX.size.y - 2):
		screen.put(SERVICE_COLUMN, TRUSTEES_BOX.position.y + 1 + index, lines[index], TEXT, BACKGROUND)


func _draw_messages(messages: Array, screen: TextScreen) -> void:
	screen.box(MESSAGES_BOX.position.x, MESSAGES_BOX.position.y, MESSAGES_BOX.size.x, MESSAGES_BOX.size.y, TEXT, BACKGROUND, MESSAGES_TITLE, false)
	var shown := messages.slice(maxi(messages.size() - MESSAGES_SHOWN, 0))
	for index in shown.size():
		var line := TextScreen.pad_right(MESSAGE_PREFIX + shown[index], MESSAGE_WIDTH)
		screen.put(COLUMN, MESSAGES_BOX.position.y + 1 + index, line, TEXT, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, FRAME, BACKGROUND, title, is_double)
