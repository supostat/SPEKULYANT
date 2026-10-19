class_name NumberDialog
extends RefCounted
## Окно ввода числа поверх экрана — количество на рынке, сумма на экране денег: заголовок, предмет,
## строка о цене или остатке, поле числа, отказ ядра и подсказка. Первая цифра набирает число
## заново, Backspace стирает, ↑↓ — ±1, Enter — подтвердить, Esc — отмена.

const AREA := Rect2i(18, 7, 44, 9)
const BACKGROUND := 1
const FOREGROUND := 15
const TEXT := 7
const FIELD_FOREGROUND := 0
const FIELD_BACKGROUND := 3
const WARNING_FOREGROUND := 14
const WARNING_ROWS := 2
const HINT_FOREGROUND := 8
const COLUMN := 3
const HINT := "Enter — %s   ↑↓ ±1   Esc — отмена"
## Чем кончилось действие ввода: подтверждено, отменено, число изменилось или ничего.
const CONFIRMED := "confirmed"
const CANCELLED := "cancelled"
const EDITED := "edited"
const IGNORED := ""

## Отказ ядра остаётся в окне до следующей правки.
var warning := ""
var _title: String
var _subject: String
var _detail: String
var _label: String
var _verb: String
var _longest: int
var _number := "1"
var _typed := false


func _init(title: String, subject: String, detail: String, label: String, verb: String, longest: int) -> void:
	_title = title
	_subject = subject
	_detail = detail
	_label = label
	_verb = verb
	_longest = longest


func number() -> int:
	return int(_number)


func handle(action: Dictionary) -> String:
	match action["action"]:
		"digit":
			var typed: String = (_number if _typed else "") + str(action["digit"])
			_number = typed.substr(0, _longest).lstrip("0")
			_typed = true
		"erase":
			_number = _number.substr(0, maxi(_number.length() - 1, 0))
			_typed = true
		"up":
			_number = str(mini(number() + 1, 10 ** _longest - 1))
		"down":
			_number = str(maxi(number() - 1, 1))
		"confirm":
			return CONFIRMED if number() > 0 else IGNORED
		"back":
			return CANCELLED
		_:
			return IGNORED
	warning = ""
	return EDITED


func draw(screen: TextScreen) -> void:
	var left := AREA.position.x + COLUMN
	var top := AREA.position.y
	screen.shadow(AREA.position.x, top, AREA.size.x, AREA.size.y)
	screen.box(AREA.position.x, top, AREA.size.x, AREA.size.y, FOREGROUND, BACKGROUND, _title)
	screen.put(left, top + 2, _subject, FOREGROUND, BACKGROUND)
	screen.put(left, top + 3, _detail, TEXT, BACKGROUND)
	screen.put(left, top + 4, _label, TEXT, BACKGROUND)
	var field := TextScreen.pad_left(_number, _longest + 2)
	screen.put(left + _label.length() + 1, top + 4, field, FIELD_FOREGROUND, FIELD_BACKGROUND)
	var lines := TextScreen.wrap(warning, AREA.size.x - 2 * COLUMN)
	for index in mini(lines.size(), WARNING_ROWS):
		screen.put(left, top + 5 + index, lines[index], WARNING_FOREGROUND, BACKGROUND)
	screen.put(left, top + 7, HINT % _verb, HINT_FOREGROUND, BACKGROUND)
