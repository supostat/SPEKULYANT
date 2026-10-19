extends SceneTree
## Оконный снимок экрана в PNG для `cargo xtask screenshot`: пиксели рисует только оконный режим.
## С флагом `--crt` поверх экрана включён CRT-эффект.

const ScreenTool := preload("res://tools/screen_tool.gd")
const FRAMES_BEFORE_CAPTURE := 3
const CRT_FLAG := "--crt"


func _initialize() -> void:
	var arguments := OS.get_cmdline_user_args()
	var crt := CRT_FLAG in arguments
	if crt:
		arguments.remove_at(arguments.find(CRT_FLAG))
	var problem := ScreenTool.usage_error(arguments)
	if not problem.is_empty():
		push_error("screenshot: " + problem)
		quit(2)
		return
	var screen := TextScreen.new()
	root.add_child(screen)
	ScreenTool.draw(arguments, screen)
	if crt:
		var overlay := Crt.new()
		overlay.visible = true
		root.add_child(overlay)
	for frame in FRAMES_BEFORE_CAPTURE:
		await process_frame
	var output_path := arguments[ScreenTool.OUTPUT_ARGUMENT]
	var saved := root.get_texture().get_image().save_png(output_path)
	if saved != OK:
		push_error("screenshot: PNG не записан в %s, код %d" % [output_path, saved])
		quit(1)
		return
	quit()
