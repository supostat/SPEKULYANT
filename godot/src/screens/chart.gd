class_name ChartScreen
extends RefCounted
## График цены товара (Enter на толкучке): цена покупки по неделям за последний год на этой
## толкучке, в номинале своей недели — тренд, колебания запаса и шоки событий; справа — цены и
## запас сейчас и сколько товара у игрока. Столбик — неделя, высота — в полклетки. ↑↓ листают
## товары толкучки, ◄► — цены этого товара по городам, Esc и F10 — назад к рынку.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(1, 2, 78, 21)
const SUBTITLE := "Цена покупки по неделям · "
const SUBTITLE_FOREGROUND := 14
const LABEL_COLUMN := 3
const LABEL_WIDTH := 10
const AXIS_COLUMN := 13
const AXIS_FOREGROUND := 8
const PLOT_LEFT := 14
const PLOT_TOP := 5
const PLOT_ROWS := 12
const PLOT_WEEKS := 52
const BAR_FOREGROUND := 11
const CURRENT_FOREGROUND := 15
const FULL_BLOCK := "█"
const HALF_BLOCK := "▄"
const AXIS_VERTICAL := "│"
const AXIS_CORNER := "└"
const AXIS_HORIZONTAL := "─"
const NO_HISTORY := "Истории ещё нет: цены записываются раз в неделю."
const MUTED_FOREGROUND := 8
const NOW_COLUMN := 68
const NOW_WIDTH := 9
const NOW_TITLE := "Сейчас"
const NOW_ROWS := [["Купить", "buy"], ["Продать", "sell"], ["Запас", "stock"], ["У тебя", "held"]]
const HINT := "↑↓ товар   ◄► цены по городам   Esc — назад"
const HINT_ROW := 21
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
const CITY_PRICES_SCREEN := "city_prices"

var _cursor := GoodCursor.new()


## `{"good": идентификатор товара}`.
func enter(arguments: Dictionary) -> void:
	_cursor.good = arguments["good"]


func draw(view: Dictionary, screen: TextScreen) -> void:
	var market: Dictionary = view["market"]
	var row := _cursor.row(market["goods"])
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, row["name"])
	screen.put(LABEL_COLUMN, BOX.position.y + 1, SUBTITLE + market["place"], SUBTITLE_FOREGROUND, BACKGROUND)
	_draw_axes(screen)
	_draw_chart(row["chart"], screen)
	_draw_now(row, screen)
	screen.put(LABEL_COLUMN, HINT_ROW, HINT, MUTED_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var goods: Array = view["market"]["goods"]
	match action["action"]:
		"up", "down":
			_cursor.move(goods, -1 if action["action"] == "up" else 1)
			return {"redraw": true}
		"left", "right":
			return {"switch": CITY_PRICES_SCREEN, "with": {"good": _cursor.row(goods)["good"]}}
		"back", "confirm":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
	return {}


func answered(_answer: Dictionary) -> void:
	pass


## Толкучки нет — в пути или в зоне товаров не видно — назад.
func follow(view: Dictionary) -> Dictionary:
	return {"pop": true} if view["market"]["goods"].is_empty() else {}


func _draw_axes(screen: TextScreen) -> void:
	for row in PLOT_ROWS:
		screen.put(AXIS_COLUMN, PLOT_TOP + row, AXIS_VERTICAL, AXIS_FOREGROUND, BACKGROUND)
	screen.put(AXIS_COLUMN, PLOT_TOP + PLOT_ROWS, AXIS_CORNER + AXIS_HORIZONTAL.repeat(PLOT_WEEKS), AXIS_FOREGROUND, BACKGROUND)


## Столбики недель справа налево от нынешней: высота — от самой низкой цены до самой высокой в
## полклетки, нынешняя неделя — ярче. Подписи — цены краёв и дни первой и последней недели.
func _draw_chart(chart: Dictionary, screen: TextScreen) -> void:
	var weeks: Array = chart["weeks"]
	if weeks.is_empty():
		screen.put(PLOT_LEFT, PLOT_TOP + floori(PLOT_ROWS / 2.0), NO_HISTORY, MUTED_FOREGROUND, BACKGROUND)
		return
	var prices: Array = weeks.filter(func(price: Variant) -> bool: return price != null)
	var low: float = prices.min()
	var high: float = prices.max()
	var levels := PLOT_ROWS * 2
	var first_column := PLOT_LEFT + PLOT_WEEKS - weeks.size()
	for index in weeks.size():
		if weeks[index] == null:
			continue
		var level := PLOT_ROWS if high == low else 1 + roundi((float(weeks[index]) - low) / (high - low) * (levels - 1))
		var foreground := CURRENT_FOREGROUND if index == weeks.size() - 1 else BAR_FOREGROUND
		_draw_bar(screen, first_column + index, level, foreground)
	var bottom := PLOT_TOP + PLOT_ROWS - 1
	screen.put(LABEL_COLUMN, PLOT_TOP, TextScreen.pad_left(chart["high"], LABEL_WIDTH), TEXT, BACKGROUND)
	screen.put(LABEL_COLUMN, bottom, TextScreen.pad_left(chart["low"], LABEL_WIDTH), TEXT, BACKGROUND)
	_draw_dates(chart, first_column, weeks.size(), screen)


## Дни недель под осью: последняя — у правого края, первая — под своим столбиком, а если там тесно,
## левее, через пробел от последней.
func _draw_dates(chart: Dictionary, first_column: int, count: int, screen: TextScreen) -> void:
	var row := PLOT_TOP + PLOT_ROWS + 1
	var to: String = chart["to"]
	var to_column := PLOT_LEFT + PLOT_WEEKS - to.length()
	screen.put(to_column, row, to, TEXT, BACKGROUND)
	if count == 1:
		return
	var from: String = chart["from"]
	var from_column := clampi(first_column, PLOT_LEFT, to_column - from.length() - 1)
	screen.put(from_column, row, from, TEXT, BACKGROUND)


## Столбик снизу вверх: целые клетки и, если высота нечётная, полклетки сверху.
func _draw_bar(screen: TextScreen, column: int, level: int, foreground: int) -> void:
	var full := floori(level / 2.0)
	var bottom := PLOT_TOP + PLOT_ROWS - 1
	for cell in full:
		screen.put(column, bottom - cell, FULL_BLOCK, foreground, BACKGROUND)
	if level % 2 == 1:
		screen.put(column, bottom - full, HALF_BLOCK, foreground, BACKGROUND)


func _draw_now(row: Dictionary, screen: TextScreen) -> void:
	screen.put(NOW_COLUMN, PLOT_TOP, NOW_TITLE, SUBTITLE_FOREGROUND, BACKGROUND)
	for index in NOW_ROWS.size():
		var item: Array = NOW_ROWS[index]
		var value: String = row[item[1]]
		var top := PLOT_TOP + 1 + index * 2
		screen.put(NOW_COLUMN, top, item[0], MUTED_FOREGROUND, BACKGROUND)
		screen.put(NOW_COLUMN, top + 1, TextScreen.pad_left(value if not value.is_empty() else "—", NOW_WIDTH), TEXT, BACKGROUND)
