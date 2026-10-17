extends SceneTree
## Безголовая проверка моста для `cargo xtask bridge-smoke`: класс ядра создаётся из GDScript,
## жизнь начинается, снимок разбирается, команда доходит до ядра и меняет снимок.

const SETTINGS := {"biography": "student", "ironman": true, "seed": 7}
const EXPECTED_CITY := "Москва"
const EXPECTED_MARKET_ROWS := 8
const SPEED_COMMAND := {"command": "set_speed", "speed": 2}


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
	print("bridge_smoke: SpeculantGame отвечает, снимок ядра разобран")
	quit()


func _fail(problem: String) -> void:
	push_error("bridge_smoke: " + problem)
	quit(1)
