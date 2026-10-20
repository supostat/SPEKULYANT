class_name Router
extends RefCounted
## Стек экранов сеанса (§13.3). Сеанс открывается заставкой в лобби — каталог игрока открыт, жизнь
## не начата; жизнь стоит на экране режима, который выбирает ядро: рынок, зона или итоги. Событие,
## которое ядро держит ожидающим, ложится поверх стека модальным окном и снимается, когда исход
## прочитан. Верхний экран рисуется на узле TextScreen, получает действия ввода и отвечает
## намерением:
##   {"command": {…}} — команда ядру; ответ вернётся экрану в answered();
##   {"push": имя, "with": {…}} — экран поверх; `with`, если есть, уходит ему в enter();
##   {"switch": имя, "with": {…}} — соседний экран на месте верхнего; `with` — как у push;
##   {"pop": true} — назад; {"home": true} — назад до экрана режима; {"redraw": true} — перерисовать;
##   {"new_life": {"biography", "ironman", "seed"}} или {"resume": слот} — начать жизнь;
##   {"lobby": имя} — в лобби, на экран `имя` поверх главного меню (или на само меню);
##   {"toggle": ключ} — переключить настройку игрока;
##   {"quit": true} — выйти из игры.
## Отказ ядра начать сеанс возвращается верхнему экрану в answered(). Реальное время двигает мир
## только под экранами, где оно идёт, — рынок, поездка, зона, — и пока экран его не держит (окно
## сделки на рынке); в остальных оно стоит. Пищалке роутер
## отдаёт мелодию заставки и бипы: событие легло поверх стека, на пейджер пришло новое сообщение,
## ядро отказало.

signal quit_requested
## Бип интерфейса — Speaker.EVENT, Speaker.PAGER или Speaker.ERROR.
signal sound_cue(cue_name: String)
## Напев — мелодия заставки тонами снимка лобби.
signal melody(tones: Array)

const BOOT_SCREEN := "boot"
const MENU_SCREEN := "main_menu"
const EVENT_SCREEN := "event"
const QUIT_SCREEN := "quit"
const TIME_SCREENS: Array[String] = ["market", "travel", "zone"]
## Экраны режимов жизни по имени режима в снимке ядра: стек жизни стоит на том, что назвало ядро.
## Итоги называются по макету дизайн-книги — «end».
const MODE_SCREENS := {"market": "market", "zone": "zone", "summary": "end"}
const DEFAULT_MODE := "market"
const QUIT_COMMAND := {"command": "quit"}

var _game: SpeculantGame
var _screen: TextScreen
var _saves: Saves
var _settings: Settings
var _stack: Array = []
var _view: Dictionary = {}
var _in_life := false
## Сообщения пейджера, которые уже прозвучали.
var _heard: Array = []


func _init(game: SpeculantGame, screen: TextScreen, saves: Saves, settings: Settings) -> void:
	_game = game
	_screen = screen
	_saves = saves
	_settings = settings


## Сеанс открывается заставкой в лобби под мелодию; если каталог игрока не открылся — под бип отказа.
## Сигналы подключаются до старта.
func start() -> void:
	_stack = [ScreenRegistry.create(BOOT_SCREEN)]
	var answer := _start(_saves.lobby())
	if answer.has("error"):
		_stack.back().answered(answer)
		sound_cue.emit(Speaker.ERROR)
		refresh()
		return
	refresh()
	melody.emit(_view["boot_melody"])


## Снимок ядра на экране — `StateView` жизни или `LobbyView` лобби — со слотами каталога.
func view() -> Dictionary:
	return _view


## Имя верхнего экрана — так его называет реестр экранов.
func top_name() -> String:
	return ScreenRegistry.name_of(_stack.back())


## Реальное время идёт: под экраном со временем, если он его сейчас не держит, ядро двигает мир, и
## экран перерисовывается, если мир изменился.
func tick(milliseconds: int) -> void:
	if not _in_life or not top_name() in TIME_SCREENS or _stack.back().holds_time():
		return
	if _game.advance(milliseconds):
		refresh()


## Окно просят закрыть: первый раз — окно выхода «Abort, Retry, Fail?», повторно — выход.
func request_quit() -> void:
	if top_name() == QUIT_SCREEN:
		_follow({"quit": true})
	else:
		_follow({"push": QUIT_SCREEN})


## Действие ввода — верхнему экрану; полный экран переключается с любого экрана.
func press(action: Dictionary) -> void:
	if action.is_empty():
		return
	if action["action"] == "fullscreen":
		_follow({"toggle": Settings.FULLSCREEN})
		return
	_follow(_stack.back().handle(action, _view))


## Свежий снимок ядра: в жизни стек встаёт на экран режима, событие ложится поверх стека или
## уходит с него, верхний экран сверяется с миром, и всё перерисовывается.
func refresh() -> void:
	_refresh(true)


func _refresh(announce_pager: bool) -> void:
	_view = JSON.parse_string(_game.view_json())
	_saves.annotate(_view)
	_settings.annotate(_view)
	if _in_life:
		_follow_mode()
		_hear_pager(announce_pager)
	var intent: Dictionary = _stack.back().follow(_view)
	if intent.is_empty():
		_draw()
	else:
		_follow(intent)


## Снимок жизни несёт экран режима; снимок лобби — нет.
static func in_life(view: Dictionary) -> bool:
	return view.has("screen")


func _follow_mode() -> void:
	var mode: String = MODE_SCREENS.get(_view["screen"], MODE_SCREENS[DEFAULT_MODE])
	if _stack.is_empty() or ScreenRegistry.name_of(_stack.front()) != mode:
		_stack = [ScreenRegistry.create(mode)]
	var event_on_top := top_name() == EVENT_SCREEN
	if _view["pending_event"] != null and not event_on_top:
		_stack.append(ScreenRegistry.create(EVENT_SCREEN))
		sound_cue.emit(Speaker.EVENT)
	elif _view["pending_event"] == null and event_on_top:
		_stack.pop_back()


## Новое последнее сообщение ленты — бип пейджера; первый снимок сеанса только запоминает ленту.
func _hear_pager(announce: bool) -> void:
	var pager: Variant = _view["pager"]
	var feed: Array = [] if pager == null else pager["messages"]
	if announce and not feed.is_empty() and not _heard.has(feed.back()):
		sound_cue.emit(Speaker.PAGER)
	_heard = feed


func _follow(intent: Dictionary) -> void:
	if intent.has("command"):
		var answer: Dictionary = JSON.parse_string(_game.apply(JSON.stringify(intent["command"])))
		if answer.has("error"):
			sound_cue.emit(Speaker.ERROR)
		_stack.back().answered(answer)
		refresh()
	elif intent.has("push"):
		_stack.append(_created(intent["push"], intent.get("with", {})))
		refresh()
	elif intent.has("switch"):
		_stack[-1] = _created(intent["switch"], intent.get("with", {}))
		refresh()
	elif intent.has("pop") and _stack.size() > 1:
		_stack.pop_back()
		refresh()
	elif intent.has("home"):
		_stack.resize(1)
		refresh()
	elif intent.has("redraw"):
		_draw()
	elif intent.has("new_life"):
		var settings: Dictionary = intent["new_life"]
		_begin_life(_saves.new_life(settings["biography"], settings["ironman"], settings["seed"]))
	elif intent.has("resume"):
		_begin_life(_saves.resume(intent["resume"]))
	elif intent.has("lobby"):
		_open_lobby(intent["lobby"])
	elif intent.has("toggle"):
		_settings.toggle(intent["toggle"])
		refresh()
	elif intent.has("quit"):
		_leave_life()
		quit_requested.emit()


func _created(screen_name: String, arguments: Dictionary) -> RefCounted:
	var created := ScreenRegistry.create(screen_name)
	if not arguments.is_empty():
		created.enter(arguments)
	return created


## Жизнь начинается на экране режима; отказ ядра остаётся на экране, откуда её начинали.
func _begin_life(start_json: String) -> void:
	var answer := _start(start_json)
	if answer.has("error"):
		_refuse(answer)
		return
	_in_life = true
	_stack.clear()
	_refresh(false)


func _open_lobby(screen_name: String) -> void:
	_leave_life()
	var answer := _start(_saves.lobby())
	if answer.has("error"):
		_refuse(answer)
		return
	_in_life = false
	_stack = [ScreenRegistry.create(MENU_SCREEN)]
	if screen_name != MENU_SCREEN:
		_stack.append(ScreenRegistry.create(screen_name))
	refresh()


## Уходя из жизни, ядро получает quit: ironman-жизнь ложится в свой слот.
func _leave_life() -> void:
	if _in_life:
		_game.apply(JSON.stringify(QUIT_COMMAND))


## Ядро не открыло сеанс: отказ — экрану, откуда его открывали, и бип.
func _refuse(answer: Dictionary) -> void:
	sound_cue.emit(Speaker.ERROR)
	_stack.back().answered(answer)
	_draw()


func _start(start_json: String) -> Dictionary:
	return JSON.parse_string(_game.new_life(start_json))


func _draw() -> void:
	_stack.back().draw(_view, _screen)
