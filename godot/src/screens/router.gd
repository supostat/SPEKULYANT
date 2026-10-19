class_name Router
extends RefCounted
## Стек экранов (§13.3): верхний рисуется на узле TextScreen и получает действия ввода. Экран отвечает
## намерением — команда ядру, экран поверх, вернуться назад или перерисоваться; ответ ядра на
## команду возвращается экрану. Событие, которое ядро держит ожидающим, ложится поверх стека
## модальным окном и снимается, когда исход прочитан.

const EVENT_SCREEN := "event"
const MARKET_SCREEN := "market"

var _game: SpeculantGame
var _screen: TextScreen
var _stack: Array = []
var _view: Dictionary = {}


func _init(game: SpeculantGame, screen: TextScreen) -> void:
	_game = game
	_screen = screen
	_stack.append(ScreenRegistry.create(MARKET_SCREEN))
	refresh()


## Снимок мира, который сейчас на экране.
func view() -> Dictionary:
	return _view


## Имя верхнего экрана — так его называет реестр экранов.
func top_name() -> String:
	return ScreenRegistry.name_of(_stack.back())


## Реальное время идёт: ядро двигает мир, и экран перерисовывается, если мир изменился.
func tick(milliseconds: int) -> void:
	if _game.advance(milliseconds):
		refresh()


## Действие ввода — верхнему экрану.
func press(action: Dictionary) -> void:
	if action.is_empty():
		return
	_follow(_stack.back().handle(action, _view))


## Свежий снимок ядра: событие встаёт поверх стека или уходит с него, верхний экран сверяется с
## миром, и всё перерисовывается.
func refresh() -> void:
	_view = JSON.parse_string(_game.view_json())
	var event_on_top := top_name() == EVENT_SCREEN
	if _view["pending_event"] != null and not event_on_top:
		_stack.append(ScreenRegistry.create(EVENT_SCREEN))
	elif _view["pending_event"] == null and event_on_top:
		_stack.pop_back()
	var intent: Dictionary = _stack.back().follow(_view)
	if intent.is_empty():
		_draw()
	else:
		_follow(intent)


func _follow(intent: Dictionary) -> void:
	if intent.has("command"):
		var answer: Dictionary = JSON.parse_string(_game.apply(JSON.stringify(intent["command"])))
		_stack.back().answered(answer)
		refresh()
	elif intent.has("push"):
		_stack.append(ScreenRegistry.create(intent["push"]))
		refresh()
	elif intent.has("pop") and _stack.size() > 1:
		_stack.pop_back()
		refresh()
	elif intent.has("redraw"):
		_draw()


func _draw() -> void:
	_stack.back().draw(_view, _screen)
