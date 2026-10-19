class_name MoneyScreen
extends RefCounted
## Деньги (docs/SPEKULYANT.md §6, §11, §13.3 п. 6): где лежат деньги, с какими рисками, курсы дня, а
## ещё жильё — место, где лежит «под матрасом», — и сам персонаж: возраст и здоровье. F2 кладёт
## в выбранное место из наличных, F3 снимает в наличные — окном суммы; F4 или Enter на жилье —
## купить его за наличные, с подтверждением.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const COLUMN := 3
const HEADING_FOREGROUND := 14
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const MUTED_FOREGROUND := 8
const PLACES_BOX := Rect2i(1, 2, 78, 10)
const PLACES_TITLE := "Деньги"
const PLACES_ROW := 4
const PLACES_VISIBLE := 6
const PLACE_WIDTH := 22
const AMOUNT_WIDTH := 14
const RISK_WIDTH := 35
const GAP := "   "
const PLACE_HEADINGS := ["Где", "Сколько", "Риск"]
const PLACES_HINT := "↑↓ выбрать   F2 положить   F3 снять"
const RATES_BOX := Rect2i(1, 12, 26, 9)
const RATES_TITLE := "Курсы"
const RUBLES_ONLY := "Только рубли."
const LIFE_BOX := Rect2i(28, 12, 51, 9)
const LIFE_TITLE := "Жильё"
const LIFE_COLUMN := 30
const NAME_FOREGROUND := 15
const LIVES_IN := "Живёшь: "
const HOMES_ROW := 16
const HOMES_VISIBLE := 3
const HOME_WIDTH := 30
const PRICE_WIDTH := 15
const HOMES_HINT := "F4 — купить выбранное"
const UNAVAILABLE_FOREGROUND := 8
const NUMBER_LABEL := "Сумма:"
const LONGEST_AMOUNT := 9
const DEALS := {
	"deposit": {"title": "Положить", "verb": "положить"},
	"withdraw": {"title": "Снять", "verb": "снять"},
}
const BALANCE_LABEL := "Лежит: "
const HOUSING_TITLE := "Жильё"
const HOUSING_QUESTION := "Купить: %s?"
const HOUSING_DETAIL := "За %s наличными; прежнее жильё уходит в зачёт."
const HOUSING_VERB := "купить"
const WARNING_ROW := 22
const WARNING_FOREGROUND := 14
const FKEYS := [
	["1", "Помощь"],
	["2", "Положить"],
	["3", "Снять"],
	["4", "Купить"],
	["5", "Связи"],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Назад"],
]
const HELP_KEY := 1
const DEPOSIT_KEY := 2
const WITHDRAW_KEY := 3
const BUY_KEY := 4
const CONTACTS_KEY := 5
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const BACK_KEY := 10

var _places := ListCursor.new(PLACES_VISIBLE)
var _homes := ListCursor.new(HOMES_VISIBLE)
## Выделение в списке жилья, а не мест для денег.
var _on_homes := false
var _warning := ""
## Окно суммы и что оно делает — `{"command", "place"}`; без окна — null.
var _amount: NumberDialog
var _deal := {}
var _purchase: ConfirmDialog


func draw(view: Dictionary, screen: TextScreen) -> void:
	var places: Array = view["money"]["places"]
	var homes := _homes_for_sale(view)
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_box(screen, PLACES_BOX, PLACES_TITLE, true)
	var headings: String = TextScreen.pad_right(PLACE_HEADINGS[0], PLACE_WIDTH) + TextScreen.pad_left(PLACE_HEADINGS[1], AMOUNT_WIDTH) + GAP + PLACE_HEADINGS[2]
	screen.put(COLUMN, PLACES_BOX.position.y + 1, headings, HEADING_FOREGROUND, BACKGROUND)
	for index: int in _places.shown(places.size()):
		var place: Dictionary = places[index]
		var line: String = TextScreen.pad_right(place["title"], PLACE_WIDTH) + TextScreen.pad_left(place["amount"], AMOUNT_WIDTH) + GAP + TextScreen.pad_right(place["risk"], RISK_WIDTH)
		_put_row(screen, COLUMN, PLACES_ROW + index - _places.first, line, TEXT, not _on_homes and index == _places.selected)
	screen.put(COLUMN, PLACES_BOX.end.y - 2, PLACES_HINT, MUTED_FOREGROUND, BACKGROUND)
	_box(screen, RATES_BOX, RATES_TITLE, false)
	var rates: Array = view["money"]["rates"]
	if rates.is_empty():
		screen.put(COLUMN, RATES_BOX.position.y + 1, RUBLES_ONLY, MUTED_FOREGROUND, BACKGROUND)
	for index in mini(rates.size(), RATES_BOX.size.y - 2):
		screen.put(COLUMN, RATES_BOX.position.y + 1 + index, rates[index], TEXT, BACKGROUND)
	_draw_life(view["character"], homes, screen)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)
	if _amount != null:
		_amount.draw(screen)
	elif _purchase != null:
		_purchase.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if _amount != null:
		return _handle_amount(action)
	if _purchase != null:
		return _handle_purchase(action, view)
	var places: Array = view["money"]["places"]
	var homes := _homes_for_sale(view)
	match action["action"]:
		"up":
			if _on_homes and _homes.selected == 0:
				_on_homes = false
				_places.select(places.size() - 1, places.size())
			elif _on_homes:
				_homes.move(-1, homes.size())
			else:
				_places.move(-1, places.size())
		"down":
			if not _on_homes and _places.selected >= places.size() - 1 and not homes.is_empty():
				_on_homes = true
				_homes.select(0, homes.size())
			elif _on_homes:
				_homes.move(1, homes.size())
			else:
				_places.move(1, places.size())
		"click":
			var place_row := _places.row_at(action["row"], PLACES_ROW, places.size())
			var home_row := _homes.row_at(action["row"], HOMES_ROW, homes.size())
			if place_row >= 0:
				_on_homes = false
				_places.select(place_row, places.size())
			elif home_row >= 0 and action["column"] >= LIFE_COLUMN:
				_on_homes = true
				_homes.select(home_row, homes.size())
			else:
				return {}
		"confirm":
			if _on_homes:
				return _offer_home(homes)
			return {}
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				DEPOSIT_KEY:
					return _open_amount("deposit", places)
				WITHDRAW_KEY:
					return _open_amount("withdraw", places)
				BUY_KEY:
					return _offer_home(homes) if _on_homes else {}
				CONTACTS_KEY:
					return {"switch": "contacts"}
				BUSINESSES_KEY:
					return {"switch": "businesses"}
				PAGER_KEY:
					return {"switch": "pager"}
				BACK_KEY:
					return {"pop": true}
			return {}
		_:
			return {}
	_warning = ""
	return {"redraw": true}


## Ответ ядра: удалось — окно закрывается, отказ — остаётся в окне.
func answered(answer: Dictionary) -> void:
	var refusal: String = answer["message"] if answer.has("error") else ""
	if _amount != null:
		_amount.warning = refusal
		if refusal.is_empty():
			_amount = null
	elif _purchase != null:
		_purchase.warning = refusal
		if refusal.is_empty():
			_purchase = null
	else:
		_warning = refusal


func follow(_view: Dictionary) -> Dictionary:
	return {}


## Жильё, которое можно купить или которое продаётся, кроме того, где живёшь.
func _homes_for_sale(view: Dictionary) -> Array:
	return view["homes"].filter(func(home: Dictionary) -> bool: return not home["current"] and home["price"] != null)


func _draw_life(character: Dictionary, homes: Array, screen: TextScreen) -> void:
	_box(screen, LIFE_BOX, LIFE_TITLE, false)
	var top := LIFE_BOX.position.y
	screen.put(LIFE_COLUMN, top + 1, character["name"], NAME_FOREGROUND, BACKGROUND)
	screen.put(LIFE_COLUMN, top + 2, "%s, %s" % [character["age"], character["health"]], TEXT, BACKGROUND)
	screen.put(LIFE_COLUMN, top + 3, LIVES_IN + character["housing"], TEXT, BACKGROUND)
	for index: int in _homes.shown(homes.size()):
		var home: Dictionary = homes[index]
		var line: String = TextScreen.pad_right(home["title"], HOME_WIDTH) + TextScreen.pad_left(home["price"], PRICE_WIDTH)
		var foreground := TEXT if home["available"] else UNAVAILABLE_FOREGROUND
		_put_row(screen, LIFE_COLUMN, HOMES_ROW + index - _homes.first, line, foreground, _on_homes and index == _homes.selected)
	screen.put(LIFE_COLUMN, LIFE_BOX.end.y - 2, HOMES_HINT, MUTED_FOREGROUND, BACKGROUND)


func _open_amount(deal: String, places: Array) -> Dictionary:
	if _on_homes or places.is_empty():
		return {}
	var place: Dictionary = places[_places.selected]
	var terms: Dictionary = DEALS[deal]
	_deal = {"command": deal, "place": place["place"]}
	_amount = NumberDialog.new(terms["title"], place["title"], BALANCE_LABEL + place["amount"], NUMBER_LABEL, terms["verb"], LONGEST_AMOUNT)
	return {"redraw": true}


func _handle_amount(action: Dictionary) -> Dictionary:
	match _amount.handle(action):
		NumberDialog.CONFIRMED:
			return {"command": {"command": _deal["command"], "place": _deal["place"], "amount": _amount.number()}}
		NumberDialog.CANCELLED:
			_amount = null
			return {"redraw": true}
		NumberDialog.EDITED:
			return {"redraw": true}
	return {}


func _offer_home(homes: Array) -> Dictionary:
	if homes.is_empty():
		return {}
	var home: Dictionary = homes[_homes.selected]
	_purchase = ConfirmDialog.new(HOUSING_TITLE, HOUSING_QUESTION % home["title"], HOUSING_DETAIL % home["price"], HOUSING_VERB)
	return {"redraw": true}


func _handle_purchase(action: Dictionary, view: Dictionary) -> Dictionary:
	match _purchase.handle(action):
		ConfirmDialog.CONFIRMED:
			var homes := _homes_for_sale(view)
			return {"command": {"command": "buy_housing", "housing": homes[_homes.selected]["housing"]}}
		ConfirmDialog.CANCELLED:
			_purchase = null
			return {"redraw": true}
	return {}


func _put_row(screen: TextScreen, column: int, row: int, line: String, foreground: int, selected: bool) -> void:
	if selected:
		screen.put(column, row, line, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
	else:
		screen.put(column, row, line, foreground, BACKGROUND)


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, FRAME, BACKGROUND, title, is_double)
