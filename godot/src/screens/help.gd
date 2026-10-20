class_name HelpScreen
extends RefCounted
## Помощь по F1 (docs/SPEKULYANT.md §13.1 п. 4 и 7): клавиши игры и где идёт время. Любая клавиша
## или щелчок закрывают окно.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const WINDOW := Rect2i(8, 2, 64, 20)
const TITLE := "Помощь"
const COLUMN := 11
const FIRST_ROW := 3
const KEY_WIDTH := 15
const KEY_FOREGROUND := 15
const KEYS := [
	["F1", "эта помощь"],
	["F2–F7", "действия экрана — подписи в нижней строке"],
	["F10, Esc", "назад; на рынке и в зоне — меню выхода"],
	["↑ ↓", "выбрать строку"],
	["◄ ►", "сменить вариант политики или режима"],
	["Enter", "подтвердить, ответить, ехать"],
	["Цифры", "выбор в событии, количество, сумма, сид"],
	["Backspace", "стереть цифру"],
	["Del", "стереть сообщение пейджера"],
	["PgUp PgDn", "листать книжку и рекорды"],
	["Пробел", "пауза"],
	["+ −", "быстрее, медленнее"],
	["Мышь", "щелчок по строке или по F-клавише"],
	["F11, Alt+Enter", "полный экран или окно"],
]
const TIME_NOTE := "Время идёт на рынке, в пути и в зоне; в остальных экранах оно стоит."
const NOTE_FOREGROUND := 14
const NOTE_WIDTH := 58
const CLOSE_HINT := "Любая клавиша — закрыть"
const HINT_FOREGROUND := 8
const FKEYS := [
	["1", "Помощь"],
	["2", ""],
	["3", ""],
	["4", ""],
	["5", ""],
	["6", ""],
	["7", ""],
	["10", "Назад"],
]


func draw(_view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, FRAME, BACKGROUND, TITLE)
	for index in KEYS.size():
		var key: Array = KEYS[index]
		screen.put(COLUMN, FIRST_ROW + index, TextScreen.dot_leader(key[0] + " ", KEY_WIDTH), KEY_FOREGROUND, BACKGROUND)
		screen.put(COLUMN + KEY_WIDTH + 1, FIRST_ROW + index, key[1], TEXT, BACKGROUND)
	var row := FIRST_ROW + KEYS.size() + 1
	for line in TextScreen.wrap(TIME_NOTE, NOTE_WIDTH):
		screen.put(COLUMN, row, line, NOTE_FOREGROUND, BACKGROUND)
		row += 1
	screen.put(COLUMN, WINDOW.position.y + WINDOW.size.y - 2, CLOSE_HINT, HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(_action: Dictionary, _view: Dictionary) -> Dictionary:
	return {"pop": true}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}
