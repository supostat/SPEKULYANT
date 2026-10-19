class_name BootScreen
extends RefCounted
## Заставка: запуск SPEKUL.EXE из стилизованной командной строки (docs/SPEKULYANT.md §13.2, макет
## «Заставка»). Любая клавиша ведёт в главное меню; если каталог игрока не открылся, заставка
## показывает отказ ядра, и клавиша ведёт к выходу.

const BACKGROUND := 0
const DEFAULT_FOREGROUND := 7
const LEFT_COLUMN := 1
const TOP_ROW := 1
const LINES: Array = [
	["C:\\>CD GAMES\\SPEKUL", 7],
	["", 7],
	["C:\\GAMES\\SPEKUL>SPEKUL.EXE", 7],
	["", 7],
	["СПЕКУЛЯНТЪ  версия 1.0", 15],
	["(с) 1993 ТОО «Перестройка-Софт»  · Все права защищены, кроме прав на товар", 8],
	["", 7],
	["Проверка памяти ............. 640K  OK", 7],
	["Расширенная память ......... 3072K  OK", 7],
	["Звук ........................ PC-speaker", 7],
	["Загрузка цен 1984 года ...... OK", 7],
	["Загрузка слухов ............. 213 шт.", 7],
	["Проверка совести ............ не найдена", 14],
	["", 7],
	["Нажмите любую клавишу, чтобы начать жить_", 10],
]
const PROBLEM_PREFIX := "Ошибка чтения диска C: "
const PROBLEM_FOREGROUND := 12
const PROBLEM_WIDTH := 78

## Отказ ядра открыть каталог игрока; пусто — лобби открыто.
var _problem := ""


func draw(_view: Dictionary, screen: TextScreen) -> void:
	screen.clear(DEFAULT_FOREGROUND, BACKGROUND)
	for index in LINES.size():
		var line: Array = LINES[index]
		screen.put(LEFT_COLUMN, TOP_ROW + index, line[0], line[1], BACKGROUND)
	var problem_row := TOP_ROW + LINES.size() + 1
	for line in TextScreen.wrap(_problem, PROBLEM_WIDTH):
		screen.put(LEFT_COLUMN, problem_row, line, PROBLEM_FOREGROUND, BACKGROUND)
		problem_row += 1


func handle(_action: Dictionary, _view: Dictionary) -> Dictionary:
	return {"push": "quit"} if not _problem.is_empty() else {"switch": "main_menu"}


func answered(answer: Dictionary) -> void:
	_problem = PROBLEM_PREFIX + answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}
