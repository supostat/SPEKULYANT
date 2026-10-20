extends SceneTree
## Сквозная жизнь для `cargo xtask e2e`: настоящие ядро, мост, роутер и экраны; подменён только
## каталог игрока — `user://e2e`. Заставка → главное меню → новая жизнь: студент, ironman, сид 7 →
## шестьдесят дней на рынке с покупками и продажами, события — первым доступным выбором → график
## цены товара за эти недели → поездка в первый доступный город с подсказкой, что там дешевле и
## дороже → цены товара по городам: Москва — какой её оставили → выход в главное меню (ironman
## сохраняется сам) и «Продолжить» из его слота: снимок ядра тот же → ещё месяц торговли → уход на
## покой → итоги жизни: заметка, слот ironman стёрт, знания легли в метапрогрессию, жизнь — в
## «Рейтинг Форбс». После каждого шага сверяются буфер экрана и снимок ядра.

const FlowDriver := preload("res://tools/flow_driver.gd")
const ROOT := "user://e2e"
const BIOGRAPHY := "student"
const SEED_KEY := KEY_7
const DAY_MILLISECONDS := 1800
const TRADING_DAYS := 60
const MORE_DAYS := 30
## Покупка — в каждый третий день, продажа — на следующий.
const TRADE_CYCLE := 3
const TRADE_GOOD := "chewing_gum"
const LONGEST_WAIT_STEPS := 4000
const CONTINUE_ITEM := 1
const NEW_LIFE_ITEM := 2
const RECORDS_ITEM := 5
## Пункты меню выхода до добровольных уходов и после них.
const ITEMS_BEFORE_EXITS := 5
const ITEMS_AFTER_EXITS := 1
const REST_ENDING := "rest"
const CHART_TITLE := "Цена покупки по неделям"
const CITY_PRICES_TITLE := "Цены по городам"
const CHEAPER_LABEL := "Там дешевле: "
const DEARER_LABEL := "Там дороже: "
## Записи при рождении и каждые семь дней: за шестьдесят дней их девять.
const MIN_CHART_WEEKS := 9
## Что роутер кладёт в снимок поверх ядра: слоты каталога и настройки.
const ANNOTATIONS := ["slots", "settings"]

var _flow: FlowDriver
var _router: Router
var _name := ""
var _buys := 0
var _sells := 0
var _events := 0


func _initialize() -> void:
	_flow = FlowDriver.new(ROOT)
	_router = _flow.router
	_router.start()
	var passed := (
		_new_life()
		and _trade_for(TRADING_DAYS)
		and _chart()
		and _travel()
		and _city_prices()
		and _quit_and_continue()
		and _trade_for(MORE_DAYS)
		and _retire()
		and _summary()
	)
	_flow.clear_directory()
	_flow.dispose()
	if passed:
		print(
			"e2e: %s — %d покупок, %d продаж, %d событий с выбором, поездка, выход и «Продолжить», покой и итоги"
			% [_name, _buys, _sells, _events]
		)
		quit()


## Заставка → главное меню → новая жизнь студента в ironman с сидом 7 — рынок Москвы, а слот
## ironman записан с рождения.
func _new_life() -> bool:
	_flow.press(KEY_ENTER)
	if _router.top_name() != "main_menu":
		return _fail("заставка не открыла главное меню")
	_flow.press(FlowDriver.DIGIT_KEYS[NEW_LIFE_ITEM])
	var biographies: Array = _router.view()["biographies"]
	for _step in biographies.find_custom(func(biography: Dictionary) -> bool: return biography["biography"] == BIOGRAPHY):
		_flow.press(KEY_DOWN)
	_flow.press(SEED_KEY)
	_flow.press(KEY_ENTER)
	var view := _router.view()
	if _router.top_name() != "market" or not view["ironman"] or view["header"]["city"] != "Москва":
		return _fail("новая ironman-жизнь студента не открыла рынок Москвы")
	if not Saves.IRONMAN_SLOT in _flow.saves.slots():
		return _fail("ironman-жизнь не легла в свой слот при рождении")
	_name = view["character"]["name"]
	return _on_market()


## `days` дней на рынке: в каждый третий день — покупка товара окном количества, на следующий —
## продажа (нечего продать — ядро откажет, и окно закроется); событие, остановившее время,
## закрывается первым доступным выбором.
func _trade_for(days: int) -> bool:
	if not _select_good():
		return false
	var lived := 0
	var date: String = _router.view()["header"]["date"]
	for _step in LONGEST_WAIT_STEPS:
		if lived >= days:
			return _on_market()
		if _router.top_name() == "event":
			if not _settle():
				return false
			continue
		if _router.top_name() != "market":
			return _fail("жизнь ушла с рынка на экран %s" % _router.top_name())
		match lived % TRADE_CYCLE:
			0:
				if _deal(KEY_F2):
					_buys += 1
			1:
				if _deal(KEY_F3):
					_sells += 1
		_router.tick(DAY_MILLISECONDS)
		var today: String = _router.view()["header"]["date"]
		if today != date:
			date = today
			lived += 1
	return _fail("за %d шагов прожито %d дней из %d" % [LONGEST_WAIT_STEPS, lived, days])


## F2 или F3 на выбранном товаре, Enter: сделка удалась — окно закрылось; отказ ядра — окно
## остаётся с ним, и Esc его закрывает.
func _deal(key: Key) -> bool:
	_flow.press(key)
	if not _flow.shows("Количество:"):
		return _fail("клавиша сделки не открыла окно количества")
	_flow.press(KEY_ENTER)
	if not _flow.shows("Количество:"):
		return true
	_flow.press(KEY_ESCAPE)
	return false


## Товар сделок — жвачка, если она есть на этой толкучке, иначе первый товар.
func _good_row() -> int:
	var goods: Array = _router.view()["market"]["goods"]
	return maxi(goods.find_custom(func(good: Dictionary) -> bool: return good["good"] == TRADE_GOOD), 0)


func _select_good() -> bool:
	var goods: Array = _router.view()["market"]["goods"]
	if goods.is_empty():
		return _fail("на толкучке нет товаров")
	for _step in goods.size():
		_flow.press(KEY_UP)
	for _step in _good_row():
		_flow.press(KEY_DOWN)
	return true


## Enter на товаре — график цены: за два месяца в истории не меньше девяти недель, ↓ — соседний
## товар, Esc — снова рынок.
func _chart() -> bool:
	var goods: Array = _router.view()["market"]["goods"]
	var row := _good_row()
	_flow.press(KEY_ENTER)
	if _router.top_name() != "chart" or not _flow.shows(CHART_TITLE) or not _flow.shows(goods[row]["name"]):
		return _fail("Enter на товаре не открыл его график:\n" + _flow.buffer())
	var weeks: Array = goods[row]["chart"]["weeks"]
	if weeks.size() < MIN_CHART_WEEKS or not _flow.shows(goods[row]["chart"]["to"]):
		return _fail("в графике %d недель вместо хотя бы %d:\n%s" % [weeks.size(), MIN_CHART_WEEKS, _flow.buffer()])
	_flow.press(KEY_DOWN)
	var next := mini(row + 1, goods.size() - 1)
	if not _flow.shows(goods[next]["name"]):
		return _fail("↓ не переключил график на соседний товар:\n" + _flow.buffer())
	_flow.press(KEY_ESCAPE)
	return _on_market()


## F4 — направления, на первом доступном — что там дешевле и дороже, Enter — в путь, перемотка до
## прибытия; события дороги решаются.
func _travel() -> bool:
	_flow.press(KEY_F4)
	var destinations: Array = _router.view()["travel"]["destinations"]
	var row := destinations.find_custom(func(destination: Dictionary) -> bool: return destination["available"])
	if _router.top_name() != "travel" or row < 0:
		return _fail("F4 не открыла поездку с доступным направлением:\n" + _flow.buffer())
	for _step in row:
		_flow.press(KEY_DOWN)
	var destination: String = destinations[row]["name"]
	if not _shows_prices_of(destinations[row]):
		return _fail("поездка не говорит, что в городе %s дешевле и дороже:\n%s" % [destination, _flow.buffer()])
	_flow.press(KEY_ENTER)
	_flow.press(KEY_ENTER)
	for _step in LONGEST_WAIT_STEPS:
		match _router.top_name():
			"market":
				break
			"event":
				if not _settle():
					return false
			_:
				_router.tick(DAY_MILLISECONDS)
	if _router.view()["header"]["city"] != destination:
		return _fail("не доехали до %s:\n%s" % [destination, _flow.buffer()])
	return _on_market()


## Под списком направлений — что в выбранном городе дешевле и дороже: подпись и первая категория.
func _shows_prices_of(destination: Dictionary) -> bool:
	for item: Array in [[CHEAPER_LABEL, destination["cheaper"]], [DEARER_LABEL, destination["dearer"]]]:
		var categories: Array = item[1]
		if not categories.is_empty() and not _flow.shows(item[0] + categories[0]):
			return false
	return true


## Enter на товаре и ► — цены по городам: здесь — сейчас, город отъезда — с днём отъезда; ◄ —
## снова график, Esc — рынок.
func _city_prices() -> bool:
	var goods: Array = _router.view()["market"]["goods"]
	var cities: Array = goods[_good_row()]["cities"]
	var left := cities.find_custom(func(city: Dictionary) -> bool: return not city["here"] and city["seen"] != null)
	_flow.press(KEY_ENTER)
	_flow.press(KEY_RIGHT)
	if _router.top_name() != "city_prices" or not _flow.shows(CITY_PRICES_TITLE):
		return _fail("► на графике не открыл цены по городам:\n" + _flow.buffer())
	if left < 0 or not _flow.shows(cities[left]["seen"]):
		return _fail("цены по городам не помнят город отъезда:\n" + _flow.buffer())
	_flow.press(KEY_LEFT)
	if _router.top_name() != "chart" or not _flow.shows(CHART_TITLE):
		return _fail("◄ не вернул график:\n" + _flow.buffer())
	_flow.press(KEY_ESCAPE)
	return _on_market()


## F10 — меню выхода, «Выйти из игры», F — в главное меню: ironman ложится в слот; «Продолжить»
## возвращает ту же жизнь — снимок ядра совпадает до байта.
func _quit_and_continue() -> bool:
	var before := _core(_router.view())
	_flow.press(KEY_F10)
	var exits: Array = _router.view()["exits"]
	_flow.press(FlowDriver.DIGIT_KEYS[ITEMS_BEFORE_EXITS + exits.size() + ITEMS_AFTER_EXITS])
	if _router.top_name() != "quit":
		return _fail("меню выхода не открыло «Abort, Retry, Fail?»:\n" + _flow.buffer())
	_flow.press(KEY_F)
	if _router.top_name() != "main_menu" or not Saves.holds_ironman(_router.view()):
		return _fail("F не вернула в главное меню с жизнью в слоте ironman")
	_flow.press(FlowDriver.DIGIT_KEYS[CONTINUE_ITEM])
	if _router.top_name() != "market":
		return _fail("«Продолжить» не вернул жизнь:\n" + _flow.buffer())
	var after := _core(_router.view())
	if after != before:
		return _fail("жизнь из слота ironman разошлась со снимком до выхода")
	return _on_market()


## F10 — меню выхода, «Уйти на покой», Enter — подтверждение: жизнь кончается итогами.
func _retire() -> bool:
	_flow.press(KEY_F10)
	var exits: Array = _router.view()["exits"]
	var rest := exits.find_custom(func(exit: Dictionary) -> bool: return exit["ending"] == REST_ENDING)
	if rest < 0 or not exits[rest]["available"]:
		return _fail("на покой уйти нельзя: %s" % [exits])
	_flow.press(FlowDriver.DIGIT_KEYS[ITEMS_BEFORE_EXITS + rest + 1])
	_flow.press(KEY_ENTER)
	if _router.top_name() != "end" or not _flow.shows("ИТОГИ ЖИЗНИ"):
		return _fail("уход на покой не открыл итоги:\n" + _flow.buffer())
	return true


## Итоги: заметка со сводкой, слот ironman стёрт, метапрогрессия записана; любая клавиша — выбор
## новой жизни, а в рекордах «Рейтинга Форбс» — эта жизнь.
func _summary() -> bool:
	var summary: Variant = _router.view()["summary"]
	if summary == null or summary["lines"].is_empty() or not _flow.shows(summary["source"]):
		return _fail("на экране итогов нет заметки со сводкой:\n" + _flow.buffer())
	if Saves.IRONMAN_SLOT in _flow.saves.slots() or not _flow.saves.has_past_lives():
		return _fail("после итогов слот ironman не стёрт или метапрогрессия не записана")
	_flow.press(KEY_ENTER)
	if _router.top_name() != "new_life":
		return _fail("итоги не повели к новой жизни:\n" + _flow.buffer())
	_flow.press(KEY_ESCAPE)
	_flow.press(FlowDriver.DIGIT_KEYS[RECORDS_ITEM])
	var forbes: Array = _router.view()["records"]["ironman"]
	if _router.top_name() != "records" or not forbes.any(func(record: Dictionary) -> bool: return record["name"] == _name):
		return _fail("жизни %s нет в «Рейтинге Форбс»:\n%s" % [_name, _flow.buffer()])
	if not _flow.shows(_name):
		return _fail("экран рекордов не показывает %s" % _name)
	return true


func _settle() -> bool:
	var problem := _flow.settle_event()
	if not problem.is_empty():
		return _fail(problem)
	_events += 1
	return true


## Рынок на экране — шапка, толкучка, ход времени — и в снимке ядра рынок.
func _on_market() -> bool:
	var view := _router.view()
	if _router.top_name() != "market" or view["screen"] != "market":
		return _fail("ожидался рынок, а на экране %s" % _router.top_name())
	if not _flow.shows(view["header"]["city"]) or not _flow.shows("Купить") or not _flow.shows("Время идёт"):
		return _fail("рынок нарисован не целиком:\n" + _flow.buffer())
	return true


## Снимок ядра без того, что роутер кладёт поверх.
func _core(view: Dictionary) -> String:
	var core := view.duplicate(true)
	for key: String in ANNOTATIONS:
		core.erase(key)
	return JSON.stringify(core, "", true)


func _fail(problem: String) -> bool:
	push_error("e2e: " + problem)
	_flow.clear_directory()
	quit(1)
	return false
