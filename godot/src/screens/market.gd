class_name MarketScreen
extends RefCounted
## Рынок, главный экран (docs/SPEKULYANT.md §13.3 п. 1, макет «Рынок» дизайн-книги): шапка, толкучка,
## карман с угрозой эпохи, слухи, сообщения, ход времени и строка F-клавиш. Стрелки выбирают товар,
## F2 и F3 открывают окно количества для покупки и продажи, пробел — пауза, +/− — скорость, F4 —
## поездка.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
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
const GOODS_VISIBLE := GOODS_HINT_ROW - GOODS_FIRST_ROW
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
const BUY_KEY := 2
const SELL_KEY := 3
const TRAVEL_KEY := 4
const SLOWEST := 1
const FASTEST := 3

const DIALOG := Rect2i(18, 7, 44, 9)
const DIALOG_FOREGROUND := 15
const DIALOG_TEXT := 7
const DIALOG_FIELD_FOREGROUND := 0
const DIALOG_FIELD_BACKGROUND := 3
const DIALOG_FIELD_WIDTH := 6
const DIALOG_WARNING_FOREGROUND := 14
const DIALOG_WARNING_ROWS := 2
const DIALOG_HINT_FOREGROUND := 8
const DIALOG_COLUMN := 3
const LONGEST_QUANTITY := 4
const DEALS := {
	"buy": {"title": "Купить", "price": "buy", "verb": "купить"},
	"sell": {"title": "Продать", "price": "sell", "verb": "продать"},
}

var selected_row := 0
var _first_row := 0
## Окно количества: сделка, товар, введённое количество и предупреждение ядра; пусто — окна нет.
var _dialog := {}


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_draw_goods(view["market"], screen)
	_draw_pocket(view["pocket"], screen)
	_draw_rumors(view["rumors"], screen)
	_draw_messages(view["messages"], screen)
	TimeIndicator.draw(view["time"], screen)
	screen.fkeys(FKEYS)
	if not _dialog.is_empty():
		_draw_dialog(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if not _dialog.is_empty():
		return _handle_dialog(action)
	var goods: Array = view["market"]["goods"]
	match action["action"]:
		"up":
			_select(selected_row - 1, goods.size())
			return {"redraw": true}
		"down":
			_select(selected_row + 1, goods.size())
			return {"redraw": true}
		"click":
			var row: int = action["row"] - GOODS_FIRST_ROW
			if action["column"] >= GOODS_COLUMN and action["column"] < GOODS_COLUMN + GOODS_LINE_WIDTH and row >= 0 and row < GOODS_VISIBLE:
				_select(_first_row + row, goods.size())
				return {"redraw": true}
		"fkey":
			match action["key"]:
				BUY_KEY:
					return _open_dialog("buy", goods)
				SELL_KEY:
					return _open_dialog("sell", goods)
				TRAVEL_KEY:
					return {"push": "travel"}
		"pause":
			return {"command": {"command": "pause", "paused": not view["time"]["paused"]}}
		"faster":
			return {"command": {"command": "set_speed", "speed": mini(int(view["time"]["speed"]) + 1, FASTEST)}}
		"slower":
			return {"command": {"command": "set_speed", "speed": maxi(int(view["time"]["speed"]) - 1, SLOWEST)}}
	return {}


## Ответ ядра на сделку: удалась — окно закрывается, отказ — его текст остаётся в окне.
func answered(answer: Dictionary) -> void:
	if _dialog.is_empty():
		return
	if answer.has("error"):
		_dialog["warning"] = answer["message"]
	else:
		_dialog = {}


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _select(row: int, count: int) -> void:
	selected_row = clampi(row, 0, maxi(count - 1, 0))
	if selected_row < _first_row:
		_first_row = selected_row
	elif selected_row >= _first_row + GOODS_VISIBLE:
		_first_row = selected_row - GOODS_VISIBLE + 1


func _open_dialog(deal: String, goods: Array) -> Dictionary:
	if goods.is_empty():
		return {}
	var row: Dictionary = goods[selected_row]
	_dialog = {
		"deal": deal,
		"good": row["good"],
		"name": row["name"],
		"price": row[DEALS[deal]["price"]],
		"quantity": "1",
		"typed": false,
		"warning": "",
	}
	return {"redraw": true}


func _handle_dialog(action: Dictionary) -> Dictionary:
	var quantity: String = _dialog["quantity"]
	match action["action"]:
		"digit":
			var typed: String = (quantity if _dialog["typed"] else "") + str(action["digit"])
			_dialog["quantity"] = typed.substr(0, LONGEST_QUANTITY).lstrip("0")
			_dialog["typed"] = true
		"erase":
			_dialog["quantity"] = quantity.substr(0, maxi(quantity.length() - 1, 0))
			_dialog["typed"] = true
		"up":
			_dialog["quantity"] = str(mini(int(quantity) + 1, 10 ** LONGEST_QUANTITY - 1))
		"down":
			_dialog["quantity"] = str(maxi(int(quantity) - 1, 1))
		"confirm":
			if int(quantity) > 0:
				return {"command": {"command": _dialog["deal"], "good": _dialog["good"], "quantity": int(quantity)}}
		"back":
			_dialog = {}
		_:
			return {}
	_dialog["warning"] = ""
	return {"redraw": true}


func _draw_goods(market: Dictionary, screen: TextScreen) -> void:
	_box(screen, GOODS_BOX, market["place"], true)
	screen.put(GOODS_COLUMN, GOODS_BOX.position.y + 1, GOODS_COLUMNS, GOODS_COLUMNS_FOREGROUND, BACKGROUND)
	var goods: Array = market["goods"]
	_select(selected_row, goods.size())
	for index in range(_first_row, mini(goods.size(), _first_row + GOODS_VISIBLE)):
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
			GOODS_FIRST_ROW + index - _first_row,
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


## Окно количества: товар, цена, поле количества, предупреждение ядра и подсказка клавиш.
func _draw_dialog(screen: TextScreen) -> void:
	var deal: Dictionary = DEALS[_dialog["deal"]]
	var left := DIALOG.position.x + DIALOG_COLUMN
	var top := DIALOG.position.y
	screen.shadow(DIALOG.position.x, top, DIALOG.size.x, DIALOG.size.y)
	screen.box(DIALOG.position.x, top, DIALOG.size.x, DIALOG.size.y, DIALOG_FOREGROUND, BACKGROUND, deal["title"])
	screen.put(left, top + 2, _dialog["name"], DIALOG_FOREGROUND, BACKGROUND)
	screen.put(left, top + 3, "Цена за единицу: " + _dialog["price"], DIALOG_TEXT, BACKGROUND)
	screen.put(left, top + 4, "Количество:", DIALOG_TEXT, BACKGROUND)
	var field := TextScreen.pad_left(_dialog["quantity"], DIALOG_FIELD_WIDTH)
	screen.put(left + 12, top + 4, field, DIALOG_FIELD_FOREGROUND, DIALOG_FIELD_BACKGROUND)
	var warning := TextScreen.wrap(_dialog["warning"], DIALOG.size.x - 2 * DIALOG_COLUMN)
	for index in mini(warning.size(), DIALOG_WARNING_ROWS):
		screen.put(left, top + 5 + index, warning[index], DIALOG_WARNING_FOREGROUND, BACKGROUND)
	var hint := "Enter — %s   ↑↓ ±1   Esc — отмена" % deal["verb"]
	screen.put(left, top + 7, hint, DIALOG_HINT_FOREGROUND, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool, frame := FRAME) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, frame, BACKGROUND, title, is_double)
