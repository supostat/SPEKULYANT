extends Node
## Корневой узел игры. Дерево собирается кодом: узел текстового экрана и текущий экран поверх него.


func _ready() -> void:
	var screen := TextScreen.new()
	add_child(screen)
	BootScreen.new().draw({}, screen)
