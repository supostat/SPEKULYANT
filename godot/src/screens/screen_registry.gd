class_name ScreenRegistry
extends RefCounted
## Экраны по именам, которыми их называют эталоны дизайн-книги, роутер и инструменты xtask.

const SCREENS := {
	"boot": preload("res://src/screens/boot.gd"),
	"main_menu": preload("res://src/screens/main_menu.gd"),
	"new_life": preload("res://src/screens/new_life.gd"),
	"slots": preload("res://src/screens/slots.gd"),
	"records": preload("res://src/screens/records.gd"),
	"notebook": preload("res://src/screens/notebook.gd"),
	"quit": preload("res://src/screens/quit.gd"),
	"help": preload("res://src/screens/help.gd"),
	"settings": preload("res://src/screens/settings.gd"),
	"market": preload("res://src/screens/market.gd"),
	"chart": preload("res://src/screens/chart.gd"),
	"city_prices": preload("res://src/screens/city_prices.gd"),
	"event": preload("res://src/screens/event.gd"),
	"travel": preload("res://src/screens/travel.gd"),
	"businesses": preload("res://src/screens/businesses.gd"),
	"manager": preload("res://src/screens/manager.gd"),
	"hire": preload("res://src/screens/hire.gd"),
	"contacts": preload("res://src/screens/contacts.gd"),
	"money": preload("res://src/screens/money.gd"),
	"pager": preload("res://src/screens/pager.gd"),
	"zone": preload("res://src/screens/zone.gd"),
	"end": preload("res://src/screens/summary.gd"),
	"game_menu": preload("res://src/screens/game_menu.gd"),
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
