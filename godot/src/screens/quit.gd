class_name QuitScreen
extends RefCounted
## Выход из игры (docs/SPEKULYANT.md §13.2): подтверждение в духе DOS — «Abort, Retry, Fail?».
## A — выйти в DOS (идущая ironman-жизнь ложится в свой слот), R — вернуться, F — из жизни в главное
## меню, а из лобби — тоже в DOS: падать дальше некуда. Буквы, Enter на строке или щелчок по ней.

const TEXT := 7
const BACKGROUND := 1
const WINDOW := Rect2i(17, 7, 46, 11)
const WINDOW_TITLE := "SPEKUL.EXE"
const WINDOW_FOREGROUND := 7
const WINDOW_BACKGROUND := 0
const COLUMN := 20
const PROBLEM_ROW := 9
const PROMPT_ROW := 10
const PROMPT := "Abort, Retry, Fail?_"
const PROMPT_FOREGROUND := 15
const FIRST_CHOICE_ROW := 12
const CHOICE_WIDTH := 40
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 7
const PROBLEMS := {
	true: "Ошибка выхода: жизнь не дожита",
	false: "Ошибка выхода: рынок будет скучать",
}
const CHOICES := {
	true: [["a", "A — выйти в DOS"], ["r", "R — вернуться к жизни"], ["f", "F — в главное меню"]],
	false: [["a", "A — выйти в DOS"], ["r", "R — вернуться в меню"], ["f", "F — всё равно выйти в DOS"]],
}
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
const HELP_KEY := 1
const BACK_KEY := 10

var _selected := 1


func draw(view: Dictionary, screen: TextScreen) -> void:
	var in_life := Router.in_life(view)
	screen.clear(TEXT, BACKGROUND)
	screen.shadow(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y)
	screen.box(WINDOW.position.x, WINDOW.position.y, WINDOW.size.x, WINDOW.size.y, WINDOW_FOREGROUND, WINDOW_BACKGROUND, WINDOW_TITLE)
	screen.put(COLUMN, PROBLEM_ROW, PROBLEMS[in_life], WINDOW_FOREGROUND, WINDOW_BACKGROUND)
	screen.put(COLUMN, PROMPT_ROW, PROMPT, PROMPT_FOREGROUND, WINDOW_BACKGROUND)
	var choices: Array = CHOICES[in_life]
	for index in choices.size():
		var selected := index == _selected
		var line := TextScreen.pad_right(choices[index][1], CHOICE_WIDTH)
		screen.put(COLUMN, FIRST_CHOICE_ROW + index, line, SELECTED_FOREGROUND if selected else WINDOW_FOREGROUND, SELECTED_BACKGROUND if selected else WINDOW_BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	var choices: Array = CHOICES[Router.in_life(view)]
	match action["action"]:
		"letter":
			for choice: Array in choices:
				if choice[0] == action["letter"]:
					return _answer(choice[0], view)
		"up":
			_selected = maxi(_selected - 1, 0)
			return {"redraw": true}
		"down":
			_selected = mini(_selected + 1, choices.size() - 1)
			return {"redraw": true}
		"confirm":
			return _answer(choices[_selected][0], view)
		"click":
			var row: int = action["row"] - FIRST_CHOICE_ROW
			if row >= 0 and row < choices.size():
				return _answer(choices[row][0], view)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
	return {}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _answer(letter: String, view: Dictionary) -> Dictionary:
	match letter:
		"r":
			return {"pop": true}
		"f":
			return {"lobby": Router.MENU_SCREEN} if Router.in_life(view) else {"quit": true}
	return {"quit": true}
