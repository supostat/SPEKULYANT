extends Node
## Корневой узел игры. Дерево собирается кодом: узел текстового экрана и роутер экранов поверх него.
## Роутер открывает сеанс в каталоге игрока и ведёт его от заставки до выхода: ввод — в действия
## экранов, реальное время — в ход мира.

const MILLISECONDS_IN_SECOND := 1000.0

var _screen: TextScreen
var _router: Router


func _ready() -> void:
	_screen = TextScreen.new()
	add_child(_screen)
	_router = Router.new(SpeculantGame.new(), _screen, Saves.new())
	_router.quit_requested.connect(get_tree().quit)


func _unhandled_input(event: InputEvent) -> void:
	_router.press(Keymap.action(event, _screen))


func _process(delta: float) -> void:
	_router.tick(roundi(delta * MILLISECONDS_IN_SECOND))
