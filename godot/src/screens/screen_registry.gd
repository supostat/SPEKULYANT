class_name ScreenRegistry
extends RefCounted
## Экраны по именам, которыми их называют эталоны дизайн-книги и инструменты xtask.

const SCREENS := {
	"boot": preload("res://src/screens/boot.gd"),
	"market": preload("res://src/screens/market.gd"),
	"event": preload("res://src/screens/event.gd"),
	"travel": preload("res://src/screens/travel.gd"),
}


static func has_screen(screen_name: String) -> bool:
	return SCREENS.has(screen_name)


static func create(screen_name: String) -> RefCounted:
	return SCREENS[screen_name].new()


## Имя экрана в реестре по его экземпляру; пусто — экран не из реестра.
static func name_of(screen: RefCounted) -> String:
	for screen_name: String in SCREENS:
		if SCREENS[screen_name] == screen.get_script():
			return screen_name
	return ""
