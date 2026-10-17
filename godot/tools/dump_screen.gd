extends SceneTree
## Безголовый сброс буфера экрана в файл формата .screen для `cargo xtask screen-check`.

const ScreenTool := preload("res://tools/screen_tool.gd")


func _initialize() -> void:
	var arguments := OS.get_cmdline_user_args()
	var problem := ScreenTool.usage_error(arguments)
	if not problem.is_empty():
		push_error("dump_screen: " + problem)
		quit(2)
		return
	var screen := TextScreen.new()
	ScreenTool.draw(arguments, screen)
	var output_path := arguments[ScreenTool.OUTPUT_ARGUMENT]
	var file := FileAccess.open(output_path, FileAccess.WRITE)
	if file == null:
		push_error("dump_screen: не открывается на запись %s" % output_path)
		screen.free()
		quit(1)
		return
	file.store_string(screen.dump())
	file.close()
	screen.free()
	quit()
