extends SceneTree
## Безголовая проверка моста для `cargo xtask bridge-smoke`: класс ядра создаётся из GDScript,
## жизнь начинается, снимок разбирается, команда доходит до ядра, advance двигает календарь,
## а покупка на толкучке кладёт товар в карман. С `--full` (`cargo xtask bridge-smoke --full`)
## мост гоняет настоящее ядро дальше: лобби каталога игрока с биографиями и без команд, тридцать
## дней с событием и выбором, покупка и продажа, поездка, сохранение, загрузка и продолжение из
## слота в каталоге игрока и каждая команда интерфейса с ответом «ок» или ошибкой с текстом для
## игрока.

const FULL_FLAG := "--full"
const NO_LIFE := "no_life"
const LIFE := {"biography": "student", "ironman": true, "seed": 7}
const EXPECTED_CITY := "Москва"
const EXPECTED_MARKET_ROWS := 31  # из сорока товаров девять появятся позже 1984 года
const SPEED_COMMAND := {"command": "set_speed", "speed": 2}
const FIRST_DAY := "1 января 1984, вс"
const SECOND_DAY := "2 января 1984, пн"
const SECOND_SPEED_DAY_MILLISECONDS := 1000
const BUY_COMMAND := {"command": "buy", "good": "chewing_gum", "quantity": 1}
const SELL_COMMAND := {"command": "sell", "good": "chewing_gum", "quantity": 1}
const BOUGHT_POCKET_LINE := {"label": "Жвачка", "amount": "1 бл"}

const SMOKE_ROOT := "user://bridge_smoke"
const FIRST_SPEED_DAY_MILLISECONDS := 1800
const DAYS_TO_LIVE := 30
const LONGEST_WAIT_STEPS := 2000
const SLOT := "smoke"
const SAVE_COMMAND := {"command": "save", "slot": SLOT}
const CHANGE_COMMAND := {"command": "set_speed", "speed": 3}
const LOAD_COMMAND := {"command": "load", "slot": SLOT}
const ARRIVAL_COMMAND := {"command": "fast_forward", "until": "until_arrival"}
## Каждая команда интерфейса — её ответ должен быть «ок» или ошибкой с видом и текстом.
const EVERY_COMMAND := [
	{"command": "new_life", "biography": "engineer", "ironman": false, "seed": 3},
	{"command": "advance", "milliseconds": 1800},
	{"command": "set_speed", "speed": 3},
	{"command": "pause", "paused": true},
	{"command": "fast_forward", "until": "until_event"},
	{"command": "buy", "good": "chewing_gum", "quantity": 1},
	{"command": "sell", "good": "chewing_gum", "quantity": 1},
	{"command": "travel", "destination": "leningrad"},
	{"command": "choose", "choice": 1},
	{"command": "continue"},
	{"command": "open_screen", "screen": "pager"},
	{"command": "open_business", "kind": "sewing_workshop"},
	{"command": "set_policy", "business": "sewing_workshop@moscow", "policy": "markup", "option": 1},
	{"command": "hire", "business": "sewing_workshop@moscow", "candidate": "gena_kabanov"},
	{"command": "fire", "business": "sewing_workshop@moscow", "employee": "gena_kabanov"},
	{"command": "call", "contact": "kofi"},
	{"command": "erase_pager_message", "message": 0},
	{"command": "buy_housing", "housing": "communal_room"},
	{"command": "retire", "ending": "rest"},
	{"command": "set_work", "work": true},
	{"command": "use_service", "service": "infirmary"},
	{"command": "order_parcel"},
	{"command": "apply_for_parole"},
	{"command": "bribe_for_release"},
	{"command": "deposit", "place": "savings_bank", "amount": 10},
	{"command": "withdraw", "place": "savings_bank", "amount": 10},
	{"command": "save", "slot": "first"},
	{"command": "load", "slot": "first"},
	{"command": "quit"},
]


func _initialize() -> void:
	if not _smoke():
		return
	if FULL_FLAG in OS.get_cmdline_user_args():
		if not _full():
			return
		print("bridge_smoke --full: тридцать дней с событием, торговля, поездка, сохранение и все команды через мост")
	quit()


func _smoke() -> bool:
	var game := SpeculantGame.new()
	if not _ok(game.new_life(JSON.stringify({"life": LIFE})), "new_life"):
		return false
	var view := _view(game)
	if view["header"]["city"] != EXPECTED_CITY:
		return _fail("город в шапке «%s», ожидался «%s»" % [view["header"]["city"], EXPECTED_CITY])
	if view["market"]["goods"].size() != EXPECTED_MARKET_ROWS:
		return _fail("строк рынка %d, ожидалось %d" % [view["market"]["goods"].size(), EXPECTED_MARKET_ROWS])
	var applied: Dictionary = JSON.parse_string(game.apply(JSON.stringify(SPEED_COMMAND)))
	var after := _view(game)
	if applied.has("error") or int(after["time"]["speed"]) != SPEED_COMMAND["speed"]:
		return _fail("команда set_speed не дошла до ядра: ответ %s, скорость %s" % [applied, after["time"]["speed"]])
	if after["header"]["date"] != FIRST_DAY:
		return _fail("жизнь началась %s, ожидалось %s" % [after["header"]["date"], FIRST_DAY])
	if not game.advance(SECOND_SPEED_DAY_MILLISECONDS):
		return _fail("advance на день второй скорости не сдвинул мир")
	var next_day: String = _view(game)["header"]["date"]
	if next_day != SECOND_DAY:
		return _fail("после advance дата %s, ожидалась %s" % [next_day, SECOND_DAY])
	if not _ok(game.apply(JSON.stringify(BUY_COMMAND)), "buy"):
		return false
	var pocket: Array = _view(game)["pocket"]["items"]
	if pocket.is_empty() or pocket[0] != BOUGHT_POCKET_LINE:
		return _fail("покупка не дошла до кармана: карман %s" % [pocket])
	print("bridge_smoke: SpeculantGame отвечает, снимок ядра разобран, время идёт, торговля работает")
	return true


func _full() -> bool:
	var saves := Saves.new(SMOKE_ROOT)
	_clear(saves)
	var game := SpeculantGame.new()
	if not _lobby(game, saves):
		return false
	var start := saves.new_life(LIFE["biography"], false, LIFE["seed"])
	if not _ok(game.new_life(start), "new_life с каталогом"):
		return false
	if not _live_days_with_an_event(game):
		return false
	if not _ok(game.apply(JSON.stringify(BUY_COMMAND)), "buy"):
		return false
	if not _ok(game.apply(JSON.stringify(SELL_COMMAND)), "sell"):
		return false
	if not _travel(game):
		return false
	if not _save_load_and_resume(game, saves):
		return false
	if not _every_command_answers():
		return false
	_clear(saves)
	return true


## Лобби: каталог открыт, снимок — биографии со стартовой открытой, команды ждут жизни.
func _lobby(game: SpeculantGame, saves: Saves) -> bool:
	if not _ok(game.new_life(saves.lobby()), "лобби"):
		return false
	var lobby := _view(game)
	var student: Array = lobby["biographies"].filter(func(biography: Dictionary) -> bool: return biography["biography"] == LIFE["biography"])
	if student.is_empty() or not student[0]["unlocked"]:
		return _fail("в лобби нет открытой биографии %s: %s" % [LIFE["biography"], lobby])
	var refused: Dictionary = JSON.parse_string(game.apply(JSON.stringify(SPEED_COMMAND)))
	if refused.get("error") != NO_LIFE:
		return _fail("в лобби команда не отвергнута: %s" % [refused])
	return true


## Тридцать дней первой скорости — и хотя бы одно событие, в котором сделан выбор.
func _live_days_with_an_event(game: SpeculantGame) -> bool:
	var days := 0
	var chose := false
	for _step in LONGEST_WAIT_STEPS:
		if days >= DAYS_TO_LIVE and chose:
			return true
		var event: Variant = _view(game)["pending_event"]
		if event != null:
			if not _settle(game, event):
				return false
			chose = true
		elif game.advance(FIRST_SPEED_DAY_MILLISECONDS):
			days += 1
	return _fail("за %d шагов прожито дней %d, событие с выбором: %s" % [LONGEST_WAIT_STEPS, days, chose])


## Поездка в первое доступное направление с перемоткой до прибытия; дорожные события решаются.
func _travel(game: SpeculantGame) -> bool:
	var destination: Variant = null
	for candidate: Dictionary in _view(game)["travel"]["destinations"]:
		if candidate["available"]:
			destination = candidate
			break
	if destination == null:
		return _fail("ни одного доступного направления из %s" % [_view(game)["header"]["city"]])
	var travel := {"command": "travel", "destination": destination["city"]}
	if not _ok(game.apply(JSON.stringify(travel)), "travel"):
		return false
	if not _ok(game.apply(JSON.stringify(ARRIVAL_COMMAND)), "fast_forward"):
		return false
	for _step in LONGEST_WAIT_STEPS:
		var view := _view(game)
		if view["travel"]["journey"] == null and view["pending_event"] == null:
			if view["header"]["city"] == destination["name"]:
				return true
			return _fail("приехали в %s вместо %s" % [view["header"]["city"], destination["name"]])
		if view["pending_event"] != null:
			if not _settle(game, view["pending_event"]):
				return false
		else:
			game.advance(FIRST_SPEED_DAY_MILLISECONDS)
	return _fail("за %d шагов не доехали до %s" % [LONGEST_WAIT_STEPS, destination["name"]])


## Слот пишет ядро в каталог игрока; загрузка возвращает тот же снимок, и продолжение из слота
## в новом экземпляре моста — тоже.
func _save_load_and_resume(game: SpeculantGame, saves: Saves) -> bool:
	if not _ok(game.apply(JSON.stringify(SAVE_COMMAND)), "save"):
		return false
	var saved := game.view_json()
	if not SLOT in saves.slots():
		return _fail("слота %s нет среди %s" % [SLOT, saves.slots()])
	if not _ok(game.apply(JSON.stringify(CHANGE_COMMAND)), "set_speed после сохранения"):
		return false
	if game.view_json() == saved:
		return _fail("скорость после сохранения не изменила снимок")
	if not _ok(game.apply(JSON.stringify(LOAD_COMMAND)), "load"):
		return false
	if game.view_json() != saved:
		return _fail("после load снимок не тот, что сохранён")
	var resumed := SpeculantGame.new()
	if not _ok(resumed.new_life(saves.resume(SLOT)), "продолжение из слота"):
		return false
	if resumed.view_json() != saved:
		return _fail("продолжение из слота показывает не тот снимок")
	return true


## Каждая команда на свежей жизни в памяти отвечает «ок» или ошибкой с видом и текстом.
func _every_command_answers() -> bool:
	for command: Dictionary in EVERY_COMMAND:
		var game := SpeculantGame.new()
		if not _ok(game.new_life(JSON.stringify({"life": LIFE})), "new_life"):
			return false
		var answer: Dictionary = JSON.parse_string(game.apply(JSON.stringify(command)))
		var refused: bool = answer.get("error") is String and answer.get("message") is String
		if answer != {"ok": true} and not refused:
			return _fail("команда %s: ответ %s" % [command["command"], answer])
	return true


## Выбор — первый доступный, затем исход прочитан.
func _settle(game: SpeculantGame, event: Dictionary) -> bool:
	if event["outcome"] == null:
		var choice := 0
		for index in event["choices"].size():
			if event["choices"][index]["available"]:
				choice = index + 1
				break
		if choice == 0:
			return _fail("в событии «%s» нет доступного выбора" % [event["title"]])
		if not _ok(game.apply(JSON.stringify({"command": "choose", "choice": choice})), "choose"):
			return false
	return _ok(game.apply(JSON.stringify({"command": "continue"})), "continue")


func _clear(saves: Saves) -> void:
	var folder := SMOKE_ROOT.path_join(Saves.SAVES_FOLDER)
	for slot in saves.slots():
		DirAccess.remove_absolute(folder.path_join(slot + "." + Saves.EXTENSION))
	if saves.has_past_lives():
		DirAccess.remove_absolute(SMOKE_ROOT.path_join(Saves.META_FILE))


func _view(game: SpeculantGame) -> Dictionary:
	return JSON.parse_string(game.view_json())


func _ok(answer: String, what: String) -> bool:
	var parsed: Dictionary = JSON.parse_string(answer)
	if parsed != {"ok": true}:
		return _fail("%s отказал: %s" % [what, parsed])
	return true


func _fail(problem: String) -> bool:
	push_error("bridge_smoke: " + problem)
	quit(1)
	return false
