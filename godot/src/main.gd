extends Node
## Корневой узел игры. Дерево собирается кодом: узел текстового экрана, CRT-эффект поверх него и
## пищалка PC-speaker; роутер экранов открывает сеанс в каталоге игрока и ведёт его от заставки до
## выхода: ввод — в действия экранов, реальное время — в ход мира, бипы и мелодия — в пищалку.
## Настройки игрока включают CRT и пищалку сразу, как их переключили.

const MILLISECONDS_IN_SECOND := 1000.0

var _screen: TextScreen
var _crt: Crt
var _speaker: Speaker
var _settings: Settings
var _router: Router


func _ready() -> void:
	_screen = TextScreen.new()
	add_child(_screen)
	_crt = Crt.new()
	add_child(_crt)
	_speaker = Speaker.new()
	add_child(_speaker)
	_settings = Settings.new()
	_settings.changed.connect(_apply_settings)
	_apply_settings()
	_router = Router.new(SpeculantGame.new(), _screen, Saves.new(), _settings)
	_router.quit_requested.connect(get_tree().quit)
	_router.sound_cue.connect(_speaker.cue)
	_router.melody.connect(_speaker.play_tones)
	_router.start()


func _unhandled_input(event: InputEvent) -> void:
	_router.press(Keymap.action(event, _screen))


func _process(delta: float) -> void:
	_router.tick(roundi(delta * MILLISECONDS_IN_SECOND))


func _apply_settings() -> void:
	_crt.visible = _settings.enabled(Settings.CRT)
	_speaker.set_enabled(_settings.enabled(Settings.SOUND))
