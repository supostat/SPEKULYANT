class_name Keymap
extends RefCounted
## Ввод (docs/SPEKULYANT.md §13.1 п. 4): клавиатура в первую очередь — F1–F10, стрелки, цифры для
## выборов, буквы по латинской подписи клавиши, пробел — пауза, +/− — скорость, Esc — назад,
## Enter — подтвердить, Backspace — стереть символ, Del — стереть строку, PgUp/PgDn — листать,
## F11 и Alt+Enter — полный экран; мышь — щелчок по строке F-клавиш или по строке окна. Экраны
## получают действия, а не клавиши.

const FUNCTION_KEYS := {
	KEY_F1: 1, KEY_F2: 2, KEY_F3: 3, KEY_F4: 4, KEY_F5: 5,
	KEY_F6: 6, KEY_F7: 7, KEY_F8: 8, KEY_F9: 9, KEY_F10: 10,
}
const NAMED_KEYS := {
	KEY_UP: "up",
	KEY_DOWN: "down",
	KEY_LEFT: "left",
	KEY_RIGHT: "right",
	KEY_SPACE: "pause",
	KEY_PLUS: "faster",
	KEY_EQUAL: "faster",
	KEY_KP_ADD: "faster",
	KEY_MINUS: "slower",
	KEY_KP_SUBTRACT: "slower",
	KEY_ESCAPE: "back",
	KEY_ENTER: "confirm",
	KEY_KP_ENTER: "confirm",
	KEY_BACKSPACE: "erase",
	KEY_DELETE: "delete",
	KEY_PAGEUP: "page_up",
	KEY_PAGEDOWN: "page_down",
	KEY_F11: "fullscreen",
}
const ENTER_KEYS := [KEY_ENTER, KEY_KP_ENTER]
const FULLSCREEN := {"action": "fullscreen"}
## Прочая клавиша — тоже действие: исход события закрывается любой клавишей.
const OTHER_KEY := {"action": "other"}


## Действие по событию ввода; пусто — событие ничего не значит (отпускание, движение мыши).
static func action(event: InputEvent, screen: TextScreen) -> Dictionary:
	if event is InputEventKey:
		return key_action(event)
	if event is InputEventMouseButton:
		return _mouse_action(event, screen)
	return {}


static func key_action(event: InputEventKey) -> Dictionary:
	if not event.pressed:
		return {}
	var keycode := event.keycode
	if keycode in ENTER_KEYS and event.alt_pressed:
		return FULLSCREEN
	if FUNCTION_KEYS.has(keycode):
		return {"action": "fkey", "key": FUNCTION_KEYS[keycode]}
	if keycode >= KEY_0 and keycode <= KEY_9:
		return {"action": "digit", "digit": keycode - KEY_0}
	if keycode >= KEY_KP_0 and keycode <= KEY_KP_9:
		return {"action": "digit", "digit": keycode - KEY_KP_0}
	if NAMED_KEYS.has(keycode):
		return {"action": NAMED_KEYS[keycode]}
	if event.echo:
		return {}
	if keycode >= KEY_A and keycode <= KEY_Z:
		return {"action": "letter", "letter": String.chr(keycode).to_lower()}
	return OTHER_KEY


static func _mouse_action(event: InputEventMouseButton, screen: TextScreen) -> Dictionary:
	if not event.pressed or event.button_index != MOUSE_BUTTON_LEFT:
		return {}
	var cell := screen.cell_at(event.position)
	if cell.x < 0:
		return {}
	if cell.y == TextScreen.FKEY_ROW:
		var key := screen.fkey_at(cell.x)
		return {"action": "fkey", "key": key} if key > 0 else {}
	return {"action": "click", "column": cell.x, "row": cell.y}
