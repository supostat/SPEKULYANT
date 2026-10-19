class_name ConfirmDialog
extends RefCounted
## Окно подтверждения поверх экрана — уволить управляющего, купить жильё, уйти из жизни: вопрос,
## пояснение, отказ ядра и подсказка. Enter — да, Esc — нет.

const AREA := Rect2i(16, 8, 48, 9)
const BACKGROUND := 1
const FOREGROUND := 15
const TEXT := 7
const WARNING_FOREGROUND := 14
const HINT_FOREGROUND := 8
const COLUMN := 3
const DETAIL_ROWS := 2
const WARNING_ROWS := 2
const HINT := "Enter — %s   Esc — отмена"
const CONFIRMED := "confirmed"
const CANCELLED := "cancelled"
const IGNORED := ""

## Отказ ядра: окно остаётся, пока игрок не отменит.
var warning := ""
var _title: String
var _question: String
var _detail: String
var _verb: String


func _init(title: String, question: String, detail: String, verb: String) -> void:
	_title = title
	_question = question
	_detail = detail
	_verb = verb


func handle(action: Dictionary) -> String:
	match action["action"]:
		"confirm":
			return CONFIRMED
		"back":
			return CANCELLED
	return IGNORED


func draw(screen: TextScreen) -> void:
	var left := AREA.position.x + COLUMN
	var top := AREA.position.y
	var width := AREA.size.x - 2 * COLUMN
	screen.shadow(AREA.position.x, top, AREA.size.x, AREA.size.y)
	screen.box(AREA.position.x, top, AREA.size.x, AREA.size.y, FOREGROUND, BACKGROUND, _title)
	screen.put(left, top + 2, TextScreen.pad_right(_question, width), FOREGROUND, BACKGROUND)
	var lines := TextScreen.wrap(_detail, width)
	for index in mini(lines.size(), DETAIL_ROWS):
		screen.put(left, top + 3 + index, lines[index], TEXT, BACKGROUND)
	var warnings := TextScreen.wrap(warning, width)
	for index in mini(warnings.size(), WARNING_ROWS):
		screen.put(left, top + 5 + index, warnings[index], WARNING_FOREGROUND, BACKGROUND)
	screen.put(left, top + 7, HINT % _verb, HINT_FOREGROUND, BACKGROUND)
