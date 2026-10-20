extends Node
## Корневой узел игры. Дерево собирается кодом: узел текстового экрана, CRT-эффект поверх него и
## пищалка PC-speaker; роутер экранов открывает сеанс в каталоге игрока и ведёт его от заставки до
## выхода: ввод — в действия экранов, реальное время — в ход мира, бипы и мелодия — в пищалку.
## Настройки игрока включают CRT, пищалку и полный экран сразу, как их переключили. Окно
## открывается с наибольшим целым увеличением кадра, которое влезает в экран: на HiDPI-экране кадр
## 720×540 пикселей иначе крошечный, а целое увеличение держит глифы VGA чёткими. Закрытие окна идёт
## через окно выхода, как и F10; перед выходом пищалка замолкает.

const MILLISECONDS_IN_SECOND := 1000.0
## Остановленный поток пищалки аудиосервер отпускает на своём такте — иначе он утечёт при выходе.
const SPEAKER_RELEASE_SECONDS := 0.1
const FRAME_WIDTH := "display/window/size/viewport_width"
const FRAME_HEIGHT := "display/window/size/viewport_height"

var _screen: TextScreen
var _crt: Crt
var _speaker: Speaker
var _settings: Settings
var _router: Router


func _ready() -> void:
	get_tree().auto_accept_quit = false
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
	_router.quit_requested.connect(_shut_down)
	_router.sound_cue.connect(_speaker.cue)
	_router.melody.connect(_speaker.play_tones)
	_router.start()


func _unhandled_input(event: InputEvent) -> void:
	_router.press(Keymap.action(event, _screen))


func _process(delta: float) -> void:
	_router.tick(roundi(delta * MILLISECONDS_IN_SECOND))


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		_router.request_quit()


func _shut_down() -> void:
	_speaker.stop()
	await get_tree().create_timer(SPEAKER_RELEASE_SECONDS).timeout
	get_tree().quit()


func _apply_settings() -> void:
	_crt.visible = _settings.enabled(Settings.CRT)
	_speaker.set_enabled(_settings.enabled(Settings.SOUND))
	if _settings.enabled(Settings.FULLSCREEN):
		DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN)
	else:
		DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_WINDOWED)
		_fit_window()


## Окно — кадр, увеличенный в наибольшее целое число раз, какое влезает в рабочую область экрана
## вместе с заголовком окна, по центру этой области.
func _fit_window() -> void:
	var frame := Vector2i(ProjectSettings.get_setting(FRAME_WIDTH), ProjectSettings.get_setting(FRAME_HEIGHT))
	var usable := DisplayServer.screen_get_usable_rect(DisplayServer.window_get_current_screen())
	var decorations := DisplayServer.window_get_size_with_decorations() - DisplayServer.window_get_size()
	var room := Vector2(usable.size - decorations)
	var zoom := maxi(floori(minf(room.x / frame.x, room.y / frame.y)), 1)
	var size := frame * zoom
	DisplayServer.window_set_size(size)
	DisplayServer.window_set_position(usable.position + (usable.size - size - decorations) / 2)
