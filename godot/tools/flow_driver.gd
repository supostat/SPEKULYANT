extends RefCounted
## Общее для безголовых сценариев игрока (`ui_flow.gd`, `e2e_life.gd`): настоящий мост и роутер
## экранов в своём каталоге игрока, клавиши через Keymap — как в игре — и буфер экрана для сверки.
## Каталог сценария очищается перед стартом и после него: так файловая система — единственное, что
## сценарий подменяет.

const DIGIT_KEYS := [KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9]
const SETTINGS_FILE := "settings.cfg"
const OUTCOME_HINT := "Любая клавиша — продолжить"

var router: Router
var saves: Saves
var screen: TextScreen
var _root: String


func _init(root: String) -> void:
	_root = root
	saves = Saves.new(root)
	clear_directory()
	screen = TextScreen.new()
	router = Router.new(SpeculantGame.new(), screen, saves, Settings.new(root.path_join(SETTINGS_FILE)))


func press(keycode: Key, alt := false) -> void:
	var event := InputEventKey.new()
	event.keycode = keycode
	event.alt_pressed = alt
	event.pressed = true
	router.press(Keymap.action(event, screen))


func shows(text: String) -> bool:
	return text in buffer()


func buffer() -> String:
	var rows := PackedStringArray()
	for row in TextScreen.ROWS:
		rows.append(screen.row_text(row))
	return "\n".join(rows)


## Событие на экране: первый доступный выбор цифрой, исход, любая клавиша. Пусто — исход прочитан,
## иначе — что пошло не так.
func settle_event() -> String:
	var choices: Array = router.view()["pending_event"]["choices"]
	var choice := choices.find_custom(func(shown: Dictionary) -> bool: return shown["available"]) + 1
	if choice < 1 or choice >= DIGIT_KEYS.size():
		return "в событии нет выбора на цифру:\n" + buffer()
	press(DIGIT_KEYS[choice])
	if not shows(OUTCOME_HINT):
		return "выбор %d не показал исход:\n%s" % [choice, buffer()]
	press(KEY_ENTER)
	return ""


## Каталог игрока сценария пуст: слоты, метапрогрессия и настройки стёрты.
func clear_directory() -> void:
	var folder := _root.path_join(Saves.SAVES_FOLDER)
	for slot in saves.slots():
		DirAccess.remove_absolute(folder.path_join(slot + "." + Saves.EXTENSION))
	for file in [Saves.META_FILE, SETTINGS_FILE]:
		if FileAccess.file_exists(_root.path_join(file)):
			DirAccess.remove_absolute(_root.path_join(file))


## Узел экрана не в дереве — его освобождает сценарий.
func dispose() -> void:
	screen.free()
