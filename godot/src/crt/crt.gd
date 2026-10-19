class_name Crt
extends ColorRect
## CRT-эффект (docs/SPEKULYANT.md §13.1 п. 6): прямоугольник во весь кадр поверх экрана с шейдером
## `crt.gdshader`, который читает уже нарисованный кадр. Мышь проходит сквозь него. Включается в
## настройках; по умолчанию выключен.

const SHADER := preload("res://src/crt/crt.gdshader")


func _init() -> void:
	var shader_material := ShaderMaterial.new()
	shader_material.shader = SHADER
	material = shader_material
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
