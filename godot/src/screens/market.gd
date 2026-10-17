class_name MarketScreen
extends RefCounted
## Рынок, главный экран (docs/SPEKULYANT.md §13.3 п. 1, макет «Рынок» дизайн-книги): шапка, толкучка,
## карман с угрозой эпохи, слухи, сообщения, ход времени и строка F-клавиш.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const TITLE := " СПЕКУЛЯНТЪ "
const TITLE_FOREGROUND := 15
const TITLE_BACKGROUND := 4
const HEADER_COLUMN := 12
const HEADER_WIDTH := 68
const HEADER_FOREGROUND := 0
const HEADER_BACKGROUND := 3
const HEADER_SEPARATOR := " │ "
const GOODS_BOX := Rect2i(1, 2, 50, 15)
const GOODS_COLUMNS := "Товар                   Купить  Продать  Есть"
const GOODS_COLUMNS_FOREGROUND := 14
const GOODS_COLUMN := 3
const GOODS_FIRST_ROW := 5
const GOODS_LINE_WIDTH := 46
const NAME_WIDTH := 22
const BUY_WIDTH := 8
const SELL_WIDTH := 9
const STOCK_WIDTH := 6
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const GOODS_HINT := "↑↓ выбрать   F2 купить   F3 продать"
const GOODS_HINT_ROW := 14
const GOODS_HINT_FOREGROUND := 8
const POCKET_BOX := Rect2i(52, 2, 27, 8)
const POCKET_TITLE := "Карман"
const POCKET_COLUMN := 54
const POCKET_LINE_WIDTH := 20
const DANGER_ROW := 8
const DANGER_FOREGROUND := 12
const DANGER_FILLED := "▓"
const DANGER_EMPTY := "░"
const RUMORS_BOX := Rect2i(52, 10, 27, 7)
const RUMORS_TITLE := "Слухи"
const RUMOR_COLUMN := 54
const RUMOR_WRAP_WIDTH := 17
const RUMOR_FOREGROUND := 11
const MESSAGES_BOX := Rect2i(1, 17, 78, 5)
const MESSAGES_TITLE := "Сообщения"
const MESSAGE_COLUMN := 3
const MESSAGES_SHOWN := 3
const MESSAGE_WIDTH := 74
const MESSAGE_PREFIX := "> "
const TIME_COLUMN := 1
const TIME_ROW := 23
const PAUSED_LINE := " ПАУЗА  (пробел)"
const PAUSED_FOREGROUND := 14
const RUNNING_PREFIX := " Время идёт  "
const RUNNING_SPEED_MARK := "►"
const RUNNING_SUFFIX := "  (+/−, пробел)"
const RUNNING_FOREGROUND := 10
const FKEYS := [
	["1", "Помощь"],
	["2", "Купить"],
	["3", "Продать"],
	["4", "Ехать"],
	["5", "Связи"],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Выход"],
]

var selected_row := 0


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	_draw_header(view["header"], screen)
	_draw_goods(view["market"], screen)
	_draw_pocket(view["pocket"], screen)
	_draw_rumors(view["rumors"], screen)
	_draw_messages(view["messages"], screen)
	_draw_time(view["time"], screen)
	screen.fkeys(FKEYS)


func _draw_header(header: Dictionary, screen: TextScreen) -> void:
	screen.put(0, 0, TITLE, TITLE_FOREGROUND, TITLE_BACKGROUND)
	var parts := PackedStringArray([header["city"], header["date"]])
	parts.append_array(PackedStringArray(header["money"]))
	var line := " " + HEADER_SEPARATOR.join(parts) + " "
	screen.put(HEADER_COLUMN, 0, TextScreen.pad_right(line, HEADER_WIDTH), HEADER_FOREGROUND, HEADER_BACKGROUND)


func _draw_goods(market: Dictionary, screen: TextScreen) -> void:
	_box(screen, GOODS_BOX, market["place"], true)
	screen.put(GOODS_COLUMN, GOODS_BOX.position.y + 1, GOODS_COLUMNS, GOODS_COLUMNS_FOREGROUND, BACKGROUND)
	var goods: Array = market["goods"]
	for index in goods.size():
		var row: Dictionary = goods[index]
		var line: String = (
			TextScreen.pad_right(row["name"], NAME_WIDTH)
			+ TextScreen.pad_left(row["buy"], BUY_WIDTH)
			+ TextScreen.pad_left(row["sell"], SELL_WIDTH)
			+ TextScreen.pad_left(row["stock"], STOCK_WIDTH)
		)
		var is_selected := index == selected_row
		screen.put(
			GOODS_COLUMN,
			GOODS_FIRST_ROW + index,
			TextScreen.pad_right(line, GOODS_LINE_WIDTH),
			SELECTED_FOREGROUND if is_selected else TEXT,
			SELECTED_BACKGROUND if is_selected else BACKGROUND
		)
	screen.put(GOODS_COLUMN, GOODS_HINT_ROW, GOODS_HINT, GOODS_HINT_FOREGROUND, BACKGROUND)


func _draw_pocket(pocket: Dictionary, screen: TextScreen) -> void:
	_box(screen, POCKET_BOX, POCKET_TITLE, true)
	var items: Array = pocket["items"]
	for index in items.size():
		var item: Dictionary = items[index]
		var label: String = item["label"]
		var amount := TextScreen.pad_left(item["amount"], POCKET_LINE_WIDTH - label.length())
		screen.put(POCKET_COLUMN, POCKET_BOX.position.y + 1 + index, label + amount, TEXT, BACKGROUND)
	if pocket["danger"] != null:
		var danger: Dictionary = pocket["danger"]
		var level := int(danger["level"])
		var bar := DANGER_FILLED.repeat(level) + DANGER_EMPTY.repeat(int(danger["scale"]) - level)
		screen.put(POCKET_COLUMN, DANGER_ROW, danger["label"] + " " + bar, DANGER_FOREGROUND, BACKGROUND)


func _draw_rumors(rumors: Array, screen: TextScreen) -> void:
	_box(screen, RUMORS_BOX, RUMORS_TITLE, false)
	var lines := PackedStringArray()
	for rumor: String in rumors:
		lines.append_array(TextScreen.wrap(rumor, RUMOR_WRAP_WIDTH))
	var visible_rows := RUMORS_BOX.size.y - 2
	for index in mini(lines.size(), visible_rows):
		screen.put(RUMOR_COLUMN, RUMORS_BOX.position.y + 1 + index, lines[index], RUMOR_FOREGROUND, BACKGROUND)


func _draw_messages(messages: Array, screen: TextScreen) -> void:
	_box(screen, MESSAGES_BOX, MESSAGES_TITLE, false, TEXT)
	var shown := messages.slice(maxi(messages.size() - MESSAGES_SHOWN, 0))
	for index in shown.size():
		var line := TextScreen.pad_right(MESSAGE_PREFIX + shown[index], MESSAGE_WIDTH)
		screen.put(MESSAGE_COLUMN, MESSAGES_BOX.position.y + 1 + index, line, TEXT, BACKGROUND)


func _draw_time(time: Dictionary, screen: TextScreen) -> void:
	if time["paused"]:
		screen.put(TIME_COLUMN, TIME_ROW, PAUSED_LINE, PAUSED_FOREGROUND, BACKGROUND)
		return
	var line := RUNNING_PREFIX + RUNNING_SPEED_MARK.repeat(int(time["speed"])) + RUNNING_SUFFIX
	screen.put(TIME_COLUMN, TIME_ROW, line, RUNNING_FOREGROUND, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool, frame := FRAME) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, frame, BACKGROUND, title, is_double)
