class_name CityPricesScreen
extends RefCounted
## Цены товара по городам (◄► на графике цены): здесь — нынешние, в городах, откуда игрок уезжал, —
## какими он видел их в день отъезда, в номинале того дня; где не бывал — прочерк. Где купить
## дешевле всего и где продать дороже всего — выделено. ↑↓ листают товары толкучки, ◄► — назад к
## графику, Esc и F10 — к рынку.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(1, 2, 78, 21)
const SUBTITLE := "Цены по городам — какими ты их видел"
const SUBTITLE_FOREGROUND := 14
const COLUMN := 3
const HEADINGS := ["Город", "Купить", "Продать", "Видел"]
const HEADINGS_ROW := 5
const HEADINGS_FOREGROUND := 14
const FIRST_ROW := 6
const NAME_WIDTH := 16
const PRICE_WIDTH := 12
const SEEN_GAP := "    "
const HERE_FOREGROUND := 15
const UNKNOWN_FOREGROUND := 8
const BEST_FOREGROUND := 10
const HERE := "сейчас"
const NEVER := "не был"
const MISSING := "—"
const LEGEND := "Зелёным — где купить дешевле и где продать дороже."
const LEGEND_ROW := 18
const MUTED_FOREGROUND := 8
const HINT := "↑↓ товар   ◄► график   Esc — назад"
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
const CHART_SCREEN := "chart"

var _cursor := GoodCursor.new()


## `{"good": идентификатор товара}`.
func enter(arguments: Dictionary) -> void:
	_cursor.good = arguments["good"]


func draw(view: Dictionary, screen: TextScreen) -> void:
	var row := _cursor.row(view["market"]["goods"])
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, row["name"])
	screen.put(COLUMN, BOX.position.y + 1, SUBTITLE, SUBTITLE_FOREGROUND, BACKGROUND)
	var headings: String = (
		TextScreen.pad_right(HEADINGS[0], NAME_WIDTH)
		+ TextScreen.pad_left(HEADINGS[1], PRICE_WIDTH)
		+ TextScreen.pad_left(HEADINGS[2], PRICE_WIDTH)
		+ SEEN_GAP
		+ HEADINGS[3]
	)
	screen.put(COLUMN, HEADINGS_ROW, headings, HEADINGS_FOREGROUND, BACKGROUND)
	var cities: Array = row["cities"]
	for index in cities.size():
		_draw_city(cities[index], FIRST_ROW + index, screen)
	screen.put(COLUMN, LEGEND_ROW, LEGEND, MUTED_FOREGROUND, BACKGROUND)
	screen.put(COLUMN, HINT_ROW, HINT, MUTED_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var goods: Array = view["market"]["goods"]
	match action["action"]:
		"up", "down":
			_cursor.move(goods, -1 if action["action"] == "up" else 1)
			return {"redraw": true}
		"left", "right":
			return {"switch": CHART_SCREEN, "with": {"good": _cursor.row(goods)["good"]}}
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


## Строка города: здесь — ярче, где игрок не бывал — тускло; лучшие цены — зелёным.
func _draw_city(city: Dictionary, row: int, screen: TextScreen) -> void:
	var seen: Variant = city["seen"]
	var foreground := TEXT
	if city["here"]:
		foreground = HERE_FOREGROUND
	elif seen == null:
		foreground = UNKNOWN_FOREGROUND
	screen.put(COLUMN, row, TextScreen.pad_right(city["name"], NAME_WIDTH), foreground, BACKGROUND)
	var buy_column := COLUMN + NAME_WIDTH
	var sell_column := buy_column + PRICE_WIDTH
	var buy_foreground := BEST_FOREGROUND if city["cheapest"] else foreground
	var sell_foreground := BEST_FOREGROUND if city["dearest"] else foreground
	screen.put(buy_column, row, TextScreen.pad_left(_price(city["buy"]), PRICE_WIDTH), buy_foreground, BACKGROUND)
	screen.put(sell_column, row, TextScreen.pad_left(_price(city["sell"]), PRICE_WIDTH), sell_foreground, BACKGROUND)
	var when: String = HERE if city["here"] else (NEVER if seen == null else seen)
	screen.put(sell_column + PRICE_WIDTH + SEEN_GAP.length(), row, when, foreground, BACKGROUND)


func _price(price: String) -> String:
	return MISSING if price.is_empty() else price
