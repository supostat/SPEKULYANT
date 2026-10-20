class_name MarketScreen
extends RefCounted
## Рынок, главный экран (docs/SPEKULYANT.md §13.3 п. 1, макет «Рынок» дизайн-книги): шапка, толкучка
## — цены, запас толкучки и сколько товара у игрока, — карман с угрозой эпохи, слухи, сообщения, ход
## времени и строка F-клавиш. Стрелки выбирают товар, Enter открывает график его цены,
## F2 и F3 открывают окно количества для покупки и продажи — с ценой первой штуки и итогом сделки,
## ведь каждая следующая штука сдвигает цену; пробел — пауза, +/− — скорость, F4 — поездка; F1 —
## помощь, F5 — связи, F6 — дела, F7 — пейджер, F10 и Esc — меню выхода. Пока открыто окно
## количества, время стоит.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const GOODS_BOX := Rect2i(1, 2, 55, 15)
const GOODS_HEADINGS := ["Товар", "Купить", "Продать", "Запас", "У тебя"]
const GOODS_COLUMNS_FOREGROUND := 14
const GOODS_COLUMN := 3
const GOODS_FIRST_ROW := 5
const NAME_WIDTH := 21
const BUY_WIDTH := 8
const SELL_WIDTH := 9
const STOCK_WIDTH := 6
const HELD_WIDTH := 7
const GOODS_LINE_WIDTH := NAME_WIDTH + BUY_WIDTH + SELL_WIDTH + STOCK_WIDTH + HELD_WIDTH
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const GOODS_HINT := "↑↓ выбрать   Enter график   F2 купить   F3 продать"
const GOODS_HINT_ROW := 14
const GOODS_HINT_FOREGROUND := 8
const GOODS_VISIBLE := GOODS_HINT_ROW - GOODS_FIRST_ROW
const POCKET_BOX := Rect2i(57, 2, 22, 8)
const POCKET_TITLE := "Карман"
const POCKET_COLUMN := 59
const POCKET_LINE_WIDTH := 18
const DANGER_ROW := 8
const DANGER_FOREGROUND := 12
const RUMORS_BOX := Rect2i(57, 10, 22, 7)
const RUMORS_TITLE := "Слухи"
const RUMOR_COLUMN := 59
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
const HELP_KEY := 1
const BUY_KEY := 2
const SELL_KEY := 3
const TRAVEL_KEY := 4
const CONTACTS_KEY := 5
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const EXIT_KEY := 10
const SLOWEST := 1
const FASTEST := 3
const LONGEST_QUANTITY := 4
const DEALS := {
	"buy": {"title": "Купить", "price": "buy", "totals": "buy_totals", "verb": "купить"},
	"sell": {"title": "Продать", "price": "sell", "totals": "sell_totals", "verb": "продать"},
}
const PRICE_LABEL := "Первая штука: "
const TOTAL_LABEL := "Итого: "
const NO_TOTAL := "—"
const QUANTITY_LABEL := "Количество:"

var _goods := ListCursor.new(GOODS_VISIBLE)
## Окно количества и сделка, которую оно подтверждает: «buy» или «sell» и товар; без окна — null.
var _dialog: NumberDialog
var _deal := {}
## Итоги сделки на 1, 2, … штук из строки товара: пока окно открыто, время стоит и они не меняются.
var _totals: Array = []


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_draw_goods(view["market"], screen)
	_draw_pocket(view["pocket"], screen)
	_draw_rumors(view["rumors"], screen)
	_draw_messages(view["messages"], screen)
	TimeIndicator.draw(view["time"], screen)
	screen.fkeys(FKEYS)
	if _dialog != null:
		_dialog.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if _dialog != null:
		return _handle_dialog(action)
	var goods: Array = view["market"]["goods"]
	match action["action"]:
		"up":
			_goods.move(-1, goods.size())
			return {"redraw": true}
		"down":
			_goods.move(1, goods.size())
			return {"redraw": true}
		"click":
			var row := _goods.row_at(action["row"], GOODS_FIRST_ROW, goods.size())
			if action["column"] >= GOODS_COLUMN and action["column"] < GOODS_COLUMN + GOODS_LINE_WIDTH and row >= 0:
				_goods.select(row, goods.size())
				return {"redraw": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BUY_KEY:
					return _open_dialog("buy", goods)
				SELL_KEY:
					return _open_dialog("sell", goods)
				TRAVEL_KEY:
					return {"push": "travel"}
				CONTACTS_KEY:
					return {"push": "contacts"}
				BUSINESSES_KEY:
					return {"push": "businesses"}
				PAGER_KEY:
					return {"push": "pager"}
				EXIT_KEY:
					return {"push": "game_menu"}
		"confirm":
			if not goods.is_empty():
				return {"push": "chart", "with": {"good": goods[_goods.selected]["good"]}}
		"back":
			return {"push": "game_menu"}
		"pause":
			return {"command": {"command": "pause", "paused": not view["time"]["paused"]}}
		"faster":
			return {"command": {"command": "set_speed", "speed": mini(int(view["time"]["speed"]) + 1, FASTEST)}}
		"slower":
			return {"command": {"command": "set_speed", "speed": maxi(int(view["time"]["speed"]) - 1, SLOWEST)}}
	return {}


## Ответ ядра на сделку: удалась — окно закрывается, отказ — его текст остаётся в окне.
func answered(answer: Dictionary) -> void:
	if _dialog == null:
		return
	if answer.has("error"):
		_dialog.warning = answer["message"]
	else:
		_dialog = null


func follow(_view: Dictionary) -> Dictionary:
	return {}


## Пока открыто окно количества, время стоит: сделка не торопит.
func holds_time() -> bool:
	return _dialog != null


func _open_dialog(deal: String, goods: Array) -> Dictionary:
	if goods.is_empty():
		return {}
	var row: Dictionary = goods[_goods.selected]
	var terms: Dictionary = DEALS[deal]
	_deal = {"command": deal, "good": row["good"]}
	_totals = row[terms["totals"]]
	_dialog = NumberDialog.new(terms["title"], row["name"], PRICE_LABEL + row[terms["price"]], QUANTITY_LABEL, terms["verb"], LONGEST_QUANTITY)
	_show_total()
	return {"redraw": true}


func _handle_dialog(action: Dictionary) -> Dictionary:
	match _dialog.handle(action):
		NumberDialog.CONFIRMED:
			return {"command": {"command": _deal["command"], "good": _deal["good"], "quantity": _dialog.number()}}
		NumberDialog.CANCELLED:
			_dialog = null
			return {"redraw": true}
		NumberDialog.EDITED:
			_show_total()
			return {"redraw": true}
	return {}


## Итог для набранного количества; больше запаса толкучки или того, что есть у игрока, — прочерк, а
## ядро объяснит отказ, когда сделку подтвердят.
func _show_total() -> void:
	var quantity := _dialog.number()
	var known := quantity >= 1 and quantity <= _totals.size()
	_dialog.total = TOTAL_LABEL + (_totals[quantity - 1] if known else NO_TOTAL)


func _draw_goods(market: Dictionary, screen: TextScreen) -> void:
	_box(screen, GOODS_BOX, market["place"], true)
	screen.put(GOODS_COLUMN, GOODS_BOX.position.y + 1, _goods_line(GOODS_HEADINGS), GOODS_COLUMNS_FOREGROUND, BACKGROUND)
	var goods: Array = market["goods"]
	for index: int in _goods.shown(goods.size()):
		var row: Dictionary = goods[index]
		var line := _goods_line([row["name"], row["buy"], row["sell"], row["stock"], row["held"]])
		var is_selected := index == _goods.selected
		screen.put(
			GOODS_COLUMN,
			GOODS_FIRST_ROW + index - _goods.first,
			TextScreen.pad_right(line, GOODS_LINE_WIDTH),
			SELECTED_FOREGROUND if is_selected else TEXT,
			SELECTED_BACKGROUND if is_selected else BACKGROUND
		)
	screen.put(GOODS_COLUMN, GOODS_HINT_ROW, GOODS_HINT, GOODS_HINT_FOREGROUND, BACKGROUND)


## Строка толкучки по колонкам: название, покупка, продажа, запас толкучки, сколько у игрока.
func _goods_line(cells: Array) -> String:
	return (
		TextScreen.pad_right(cells[0], NAME_WIDTH)
		+ TextScreen.pad_left(cells[1], BUY_WIDTH)
		+ TextScreen.pad_left(cells[2], SELL_WIDTH)
		+ TextScreen.pad_left(cells[3], STOCK_WIDTH)
		+ TextScreen.pad_left(cells[4], HELD_WIDTH)
	)


## Строка кармана — подпись и количество; длинная подпись укорачивается, количество — нет.
func _draw_pocket(pocket: Dictionary, screen: TextScreen) -> void:
	_box(screen, POCKET_BOX, POCKET_TITLE, true)
	var items: Array = pocket["items"]
	for index in items.size():
		var item: Dictionary = items[index]
		var amount: String = item["amount"]
		var line := TextScreen.pad_right(item["label"], POCKET_LINE_WIDTH - amount.length() - 1) + " " + amount
		screen.put(POCKET_COLUMN, POCKET_BOX.position.y + 1 + index, line, TEXT, BACKGROUND)
	if pocket["danger"] != null:
		var danger: Dictionary = pocket["danger"]
		var bar := TextScreen.bar(int(danger["level"]), int(danger["scale"]))
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


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool, frame := FRAME) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, frame, BACKGROUND, title, is_double)
