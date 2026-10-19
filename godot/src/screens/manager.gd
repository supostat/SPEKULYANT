class_name ManagerScreen
extends RefCounted
## Управляющий (docs/SPEKULYANT.md §7.3, §13.3 п. 4, макет «Управляющий»): политики дела, черты и
## привычки управляющего, прогноз на месяц — ядро пересчитывает его после каждой политики. Экран
## открывается без выделения; ↑↓ выбирают политику, ◄ ► меняют вариант, щелчок по стрелке — тоже.
## F2 — нанять, F3 — уволить, F4 — касса (деньги), F5 — отчёт за месяц, F6 — к списку дел,
## F7 — пейджер.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const MAIN_BOX := Rect2i(1, 2, 52, 17)
const COLUMN := 3
const MANAGER_ROW := 4
const MANAGER_LINE := "Управляющий: %s, %s"
const NO_MANAGER := "Управляющего нет — дело стоит. F2 — нанять."
const MANAGER_FOREGROUND := 14
const POLICY_ROW := 6
const POLICY_LABEL_WIDTH := 20
const OPTION_COLUMN := 24
const OPTION_WIDTH := 20
const PREVIOUS := "◄ "
const NEXT := " ►"
const OPTION_FOREGROUND := 15
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const POLICY_ROWS := 12
const TRAITS_BOX := Rect2i(54, 2, 25, 9)
const TRAITS_COLUMN := 56
const TRAIT_LABEL_WIDTH := 11
const HABIT_FOREGROUND := 8
const VACANCY_TITLE := "Управляющий"
const VACANCY := "Никого."
const FORECAST_BOX := Rect2i(54, 11, 25, 8)
const FORECAST_TITLE := "Прогноз на месяц"
const REVENUE_LABEL := "Выручка  "
const RAID_LABEL := "Риск наезда:"
const THEFT_LABEL := "Риск воровства:"
const GOOD_FOREGROUND := 10
const ALARM_FOREGROUND := 12
const RISK_FOREGROUND := 14
const NO_FORECAST := ["Без управляющего", "прогноза нет."]
const HINT := " События придут, только если ситуация выйдет за эти правила."
const HINT_ROW := 20
const HINT_FOREGROUND := 8
const CLOSED_ROW := 21
const CLOSED_PREFIX := " Дело "
const WARNING_ROW := 22
const WARNING_FOREGROUND := 14
const REPORT_WINDOW := Rect2i(14, 5, 52, 14)
const REPORT_TITLE := "Отчёт за %s"
const REPORT_EMPTY_TITLE := "Отчёт"
const REPORT_EMPTY := "Отчёта ещё нет: месяц не кончился."
const REPORT_LABEL_WIDTH := 24
const REPORT_HINT := "Любая клавиша — закрыть"
const FIRE_TITLE := "Уволить"
const FIRE_QUESTION := "Уволить: %s?"
const FIRE_DETAIL := "Дело встанет, пока не наймёшь другого."
const FIRE_VERB := "уволить"
const FKEYS := [
	["1", "Помощь"],
	["2", "Нанять"],
	["3", "Уволить"],
	["4", "Касса"],
	["5", "Отчёт"],
	["6", "Дела"],
	["7", "Пейджер"],
	["10", "Назад"],
]
const HELP_KEY := 1
const HIRE_KEY := 2
const FIRE_KEY := 3
const CASH_KEY := 4
const REPORT_KEY := 5
const BUSINESSES_KEY := 6
const PAGER_KEY := 7
const BACK_KEY := 10

## Дело на экране — «тип@город»; пусто — первое из дел игрока.
var _business := ""
## Выделенная политика; −1 — экран открыт без выделения.
var _policy := -1
var _warning := ""
var _report_open := false
var _fire: ConfirmDialog


## `{"business": "тип@город"}`.
func enter(arguments: Dictionary) -> void:
	_business = arguments["business"]


func draw(view: Dictionary, screen: TextScreen) -> void:
	var business := _shown(view)
	screen.clear(TEXT, BACKGROUND)
	Header.draw(view["header"], screen)
	_box(screen, MAIN_BOX, business["title"], true)
	var manager: Variant = business["manager"]
	if manager != null:
		screen.put(COLUMN, MANAGER_ROW, MANAGER_LINE % [manager["name"], manager["since"]], MANAGER_FOREGROUND, BACKGROUND)
	else:
		screen.put(COLUMN, MANAGER_ROW, NO_MANAGER, MANAGER_FOREGROUND, BACKGROUND)
	_draw_policies(business["policies"], screen)
	_draw_manager(manager, screen)
	_draw_forecast(business["forecast"], screen)
	screen.put(1, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	if business["closed_until"] != null:
		screen.put(1, CLOSED_ROW, CLOSED_PREFIX + business["closed_until"], ALARM_FOREGROUND, BACKGROUND)
	screen.put(1, WARNING_ROW, _warning, WARNING_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)
	if _report_open:
		_draw_report(business["report"], screen)
	elif _fire != null:
		_fire.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var business := _shown(view)
	if _report_open:
		_report_open = false
		return {"redraw": true}
	if _fire != null:
		return _handle_fire(action, business)
	var policies: Array = business["policies"]
	match action["action"]:
		"up":
			_policy = maxi(_policy - 1, 0)
			return {"redraw": true}
		"down":
			_policy = mini(_policy + 1, policies.size() - 1)
			return {"redraw": true}
		"left":
			return _turn(business, -1)
		"right":
			return _turn(business, 1)
		"click":
			return _click(action, business)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				HIRE_KEY:
					return {"push": "hire", "with": {"business": business["business"]}}
				FIRE_KEY:
					if business["manager"] != null:
						_fire = ConfirmDialog.new(FIRE_TITLE, FIRE_QUESTION % business["manager"]["name"], FIRE_DETAIL, FIRE_VERB)
						return {"redraw": true}
				CASH_KEY:
					return {"switch": "money"}
				REPORT_KEY:
					_report_open = true
					return {"redraw": true}
				BUSINESSES_KEY, BACK_KEY:
					return {"pop": true}
				PAGER_KEY:
					return {"switch": "pager"}
	return {}


## Отказ ядра — под окном; удачное увольнение закрывает окно подтверждения.
func answered(answer: Dictionary) -> void:
	if _fire != null:
		if answer.has("error"):
			_fire.warning = answer["message"]
		else:
			_fire = null
		return
	_warning = answer["message"] if answer.has("error") else ""


## Дела, которое было на экране, больше нет — назад к списку.
func follow(view: Dictionary) -> Dictionary:
	return {"pop": true} if _shown(view).is_empty() else {}


func _shown(view: Dictionary) -> Dictionary:
	return business_in(view, _business)


## Дело снимка по «тип@город»; пустой идентификатор — первое дело; пусто — такого дела нет.
static func business_in(view: Dictionary, business_id: String) -> Dictionary:
	var businesses: Array = view["businesses"]
	if businesses.is_empty():
		return {}
	if business_id.is_empty():
		return businesses[0]
	var index := businesses.find_custom(func(business: Dictionary) -> bool: return business["business"] == business_id)
	return businesses[index] if index >= 0 else {}


## Ряды политик: по две строки на политику, пока хватает окна.
func _policy_step(count: int) -> int:
	return 2 if count * 2 <= POLICY_ROWS else 1


func _draw_policies(policies: Array, screen: TextScreen) -> void:
	var step := _policy_step(policies.size())
	for index in mini(policies.size(), POLICY_ROWS):
		var policy: Dictionary = policies[index]
		var row := POLICY_ROW + index * step
		var label := TextScreen.dot_leader(policy["label"], POLICY_LABEL_WIDTH)
		if index == _policy:
			screen.put(COLUMN, row, label, SELECTED_FOREGROUND, SELECTED_BACKGROUND)
		else:
			screen.put(COLUMN, row, label, TEXT, BACKGROUND)
		var option: String = policy["options"][int(policy["selected"])]
		screen.put(OPTION_COLUMN, row, PREVIOUS + TextScreen.pad_right(option, OPTION_WIDTH) + NEXT, OPTION_FOREGROUND, BACKGROUND)


func _draw_manager(manager: Variant, screen: TextScreen) -> void:
	if manager == null:
		_box(screen, TRAITS_BOX, VACANCY_TITLE, false)
		screen.put(TRAITS_COLUMN, TRAITS_BOX.position.y + 1, VACANCY, HABIT_FOREGROUND, BACKGROUND)
		return
	var name: String = manager["name"]
	_box(screen, TRAITS_BOX, name.get_slice(" ", 0), false)
	var row := TRAITS_BOX.position.y + 1
	for level: Dictionary in manager["traits"]:
		var bar := TextScreen.bar(int(level["level"]), int(level["scale"]))
		screen.put(TRAITS_COLUMN, row, TextScreen.pad_right(level["label"], TRAIT_LABEL_WIDTH) + bar, TEXT, BACKGROUND)
		row += 1
	row += 1
	for habit: String in manager["habits"]:
		if row >= TRAITS_BOX.end.y - 1:
			break
		screen.put(TRAITS_COLUMN, row, habit, HABIT_FOREGROUND, BACKGROUND)
		row += 1


func _draw_forecast(forecast: Variant, screen: TextScreen) -> void:
	_box(screen, FORECAST_BOX, FORECAST_TITLE, false)
	var top := FORECAST_BOX.position.y
	if forecast == null:
		for index in NO_FORECAST.size():
			screen.put(TRAITS_COLUMN, top + 1 + index, NO_FORECAST[index], HABIT_FOREGROUND, BACKGROUND)
		return
	screen.put(TRAITS_COLUMN, top + 1, REVENUE_LABEL + forecast["revenue"], ALARM_FOREGROUND if forecast["revenue_falls"] else GOOD_FOREGROUND, BACKGROUND)
	screen.put(TRAITS_COLUMN, top + 3, RAID_LABEL, TEXT, BACKGROUND)
	screen.put(TRAITS_COLUMN, top + 4, forecast["raid_risk"], ALARM_FOREGROUND if forecast["raid_risk_high"] else RISK_FOREGROUND, BACKGROUND)
	screen.put(TRAITS_COLUMN, top + 5, THEFT_LABEL, TEXT, BACKGROUND)
	screen.put(TRAITS_COLUMN, top + 6, forecast["theft_risk"], ALARM_FOREGROUND if forecast["theft_risk_high"] else RISK_FOREGROUND, BACKGROUND)


## Отчёт управляющего за прошлый месяц — окном поверх.
func _draw_report(report: Variant, screen: TextScreen) -> void:
	var title := REPORT_EMPTY_TITLE if report == null else REPORT_TITLE % report["month"]
	var left := REPORT_WINDOW.position.x + 3
	var top := REPORT_WINDOW.position.y
	screen.shadow(REPORT_WINDOW.position.x, top, REPORT_WINDOW.size.x, REPORT_WINDOW.size.y)
	_box(screen, REPORT_WINDOW, title, true)
	if report == null:
		screen.put(left, top + 2, REPORT_EMPTY, TEXT, BACKGROUND)
	else:
		var lines: Array = report["lines"]
		for index in mini(lines.size(), REPORT_WINDOW.size.y - 5):
			var line: Dictionary = lines[index]
			screen.put(left, top + 2 + index, TextScreen.dot_leader(line["label"] + " ", REPORT_LABEL_WIDTH) + " " + line["value"], TEXT, BACKGROUND)
	screen.put(left, top + REPORT_WINDOW.size.y - 2, REPORT_HINT, HINT_FOREGROUND, BACKGROUND)


## ◄ ► меняют вариант выделенной политики по кругу; без выделения первая стрелка выделяет первую.
func _turn(business: Dictionary, step: int) -> Dictionary:
	var policies: Array = business["policies"]
	if policies.is_empty():
		return {}
	if _policy < 0:
		_policy = 0
		return {"redraw": true}
	var policy: Dictionary = policies[_policy]
	var option := posmod(int(policy["selected"]) + step, policy["options"].size())
	return {"command": {"command": "set_policy", "business": business["business"], "policy": policy["policy"], "option": option}}


## Щелчок по строке политики выделяет её, по стрелке — ещё и меняет вариант.
func _click(action: Dictionary, business: Dictionary) -> Dictionary:
	var policies: Array = business["policies"]
	var step := _policy_step(policies.size())
	var clicked := range(policies.size()).find_custom(func(index: int) -> bool: return POLICY_ROW + index * step == action["row"])
	if clicked < 0:
		return {}
	_policy = clicked
	var column: int = action["column"]
	if column == OPTION_COLUMN:
		return _turn(business, -1)
	if column == OPTION_COLUMN + PREVIOUS.length() + OPTION_WIDTH + NEXT.length() - 1:
		return _turn(business, 1)
	return {"redraw": true}


func _handle_fire(action: Dictionary, business: Dictionary) -> Dictionary:
	match _fire.handle(action):
		ConfirmDialog.CONFIRMED:
			return {"command": {"command": "fire", "business": business["business"], "employee": business["manager"]["employee"]}}
		ConfirmDialog.CANCELLED:
			_fire = null
			return {"redraw": true}
	return {}


func _box(screen: TextScreen, area: Rect2i, title: String, is_double: bool) -> void:
	screen.box(area.position.x, area.position.y, area.size.x, area.size.y, FRAME, BACKGROUND, title, is_double)
