extends Node
## Корневой узел игры. Дерево собирается кодом: узел текстового экрана и экраны поверх него. Заставка
## ждёт любую клавишу, затем начинается жизнь в каталоге игрока, и стек экранов ведёт её: ввод — в
## действия экранов, реальное время — в ход мира.

## Старт до экрана выбора биографии и режима: студент, ironman, сид — случайный.
const START_BIOGRAPHY := "student"
const START_IRONMAN := true
const MILLISECONDS_IN_SECOND := 1000.0

var _screen: TextScreen
var _router: Router


func _ready() -> void:
	_screen = TextScreen.new()
	add_child(_screen)
	BootScreen.new().draw({}, _screen)


func _unhandled_input(event: InputEvent) -> void:
	var action := Keymap.action(event, _screen)
	if action.is_empty():
		return
	if _router == null:
		_start_life()
	else:
		_router.press(action)


func _process(delta: float) -> void:
	if _router != null:
		_router.tick(roundi(delta * MILLISECONDS_IN_SECOND))


func _start_life() -> void:
	var game := SpeculantGame.new()
	var start := Saves.new().new_life(START_BIOGRAPHY, START_IRONMAN, randi())
	var answer: Dictionary = JSON.parse_string(game.new_life(start))
	if answer.has("error"):
		push_error("жизнь не начинается: %s" % answer["message"])
		return
	_router = Router.new(game, _screen)
