class_name ScreenRegistry
extends RefCounted
## Экраны по именам, которыми их называют эталоны дизайн-книги и инструменты xtask.

const SCREENS := {
	"boot": preload("res://src/screens/boot.gd"),
	"market": preload("res://src/screens/market.gd"),
}


static func has_screen(screen_name: String) -> bool:
	return SCREENS.has(screen_name)


static func create(screen_name: String) -> RefCounted:
	return SCREENS[screen_name].new()
