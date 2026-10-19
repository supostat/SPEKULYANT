extends RefCounted
## Общее для инструментов экранов: аргументы «<экран> <выходной файл> [фикстура StateView]» и отрисовка.
## Слоты сохранений для меню берутся из пустого каталога — буфер не зависит от сохранений игрока.

const SCREEN_ARGUMENT := 0
const OUTPUT_ARGUMENT := 1
const VIEW_ARGUMENT := 2
const EMPTY_SAVES_ROOT := "user://screen_tool"


static func usage_error(arguments: PackedStringArray) -> String:
	if arguments.size() < 2 or arguments.size() > 3:
		return "ожидаются <экран> <выходной файл> [фикстура StateView], получено %s" % [arguments]
	if not ScreenRegistry.has_screen(arguments[SCREEN_ARGUMENT]):
		return "неизвестный экран «%s»" % arguments[SCREEN_ARGUMENT]
	return ""


static func draw(arguments: PackedStringArray, screen: TextScreen) -> void:
	var view: Dictionary = {}
	if arguments.size() > VIEW_ARGUMENT:
		view = JSON.parse_string(FileAccess.get_file_as_string(arguments[VIEW_ARGUMENT]))
	Saves.new(EMPTY_SAVES_ROOT).annotate(view)
	ScreenRegistry.create(arguments[SCREEN_ARGUMENT]).draw(view, screen)
