extends SceneTree
## Оконный снимок экрана в PNG для `cargo xtask screenshot`: пиксели рисует только оконный режим.

const ScreenTool := preload("res://tools/screen_tool.gd")
const FRAMES_BEFORE_CAPTURE := 3


func _initialize() -> void:
	var arguments := OS.get_cmdline_user_args()
	var problem := ScreenTool.usage_error(arguments)
	if not problem.is_empty():
		push_error("screenshot: " + problem)
		quit(2)
		return
	var screen := TextScreen.new()
	root.add_child(screen)
	ScreenTool.draw(arguments, screen)
	for frame in FRAMES_BEFORE_CAPTURE:
		await process_frame
	var output_path := arguments[ScreenTool.OUTPUT_ARGUMENT]
	var saved := root.get_texture().get_image().save_png(output_path)
	if saved != OK:
		push_error("screenshot: PNG не записан в %s, код %d" % [output_path, saved])
		quit(1)
		return
	quit()
