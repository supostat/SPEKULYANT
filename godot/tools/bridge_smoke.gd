extends SceneTree
## Безголовая проверка моста для `cargo xtask bridge-smoke`: класс ядра создаётся из GDScript,
## жизнь начинается, снимок разбирается, команда доходит до ядра, advance двигает календарь,
## а покупка на толкучке кладёт товар в карман.

const SETTINGS := {"biography": "student", "ironman": true, "seed": 7}
const EXPECTED_CITY := "Москва"
const EXPECTED_MARKET_ROWS := 31  # из сорока товаров девять появятся позже 1984 года
const SPEED_COMMAND := {"command": "set_speed", "speed": 2}
const FIRST_DAY := "1 января 1984, вс"
const SECOND_DAY := "2 января 1984, пн"
const SECOND_SPEED_DAY_MILLISECONDS := 1000
const BUY_COMMAND := {"command": "buy", "good": "chewing_gum", "quantity": 1}
const BOUGHT_POCKET_LINE := {"label": "Жвачка", "amount": "1 бл"}


func _initialize() -> void:
	var game := SpeculantGame.new()
	var started: Dictionary = JSON.parse_string(game.new_life(JSON.stringify(SETTINGS)))
	if started.has("error"):
		_fail("new_life отказал: %s" % [started])
		return
	var view: Dictionary = JSON.parse_string(game.view_json())
	if view["header"]["city"] != EXPECTED_CITY:
		_fail("город в шапке «%s», ожидался «%s»" % [view["header"]["city"], EXPECTED_CITY])
		return
	if view["market"]["goods"].size() != EXPECTED_MARKET_ROWS:
		_fail("строк рынка %d, ожидалось %d" % [view["market"]["goods"].size(), EXPECTED_MARKET_ROWS])
		return
	var applied: Dictionary = JSON.parse_string(game.apply(JSON.stringify(SPEED_COMMAND)))
	var after: Dictionary = JSON.parse_string(game.view_json())
	if applied.has("error") or int(after["time"]["speed"]) != SPEED_COMMAND["speed"]:
		_fail("команда set_speed не дошла до ядра: ответ %s, скорость %s" % [applied, after["time"]["speed"]])
		return
	if after["header"]["date"] != FIRST_DAY:
		_fail("жизнь началась %s, ожидалось %s" % [after["header"]["date"], FIRST_DAY])
		return
	if not game.advance(SECOND_SPEED_DAY_MILLISECONDS):
		_fail("advance на день второй скорости не сдвинул мир")
		return
	var next_day: String = JSON.parse_string(game.view_json())["header"]["date"]
	if next_day != SECOND_DAY:
		_fail("после advance дата %s, ожидалась %s" % [next_day, SECOND_DAY])
		return
	var bought: Dictionary = JSON.parse_string(game.apply(JSON.stringify(BUY_COMMAND)))
	var pocket: Array = JSON.parse_string(game.view_json())["pocket"]["items"]
	if bought.has("error") or pocket.is_empty() or pocket[0] != BOUGHT_POCKET_LINE:
		_fail("покупка не дошла до кармана: ответ %s, карман %s" % [bought, pocket])
		return
	print("bridge_smoke: SpeculantGame отвечает, снимок ядра разобран, время идёт, торговля работает")
	quit()


func _fail(problem: String) -> void:
	push_error("bridge_smoke: " + problem)
	quit(1)
