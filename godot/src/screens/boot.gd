class_name BootScreen
extends RefCounted
## Заставка: запуск SPEKUL.EXE из стилизованной командной строки (docs/SPEKULYANT.md §13.2, макет «Заставка»).

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


func draw(_view: Dictionary, screen: TextScreen) -> void:
	screen.clear(DEFAULT_FOREGROUND, BACKGROUND)
	for index in LINES.size():
		var line: Array = LINES[index]
		screen.put(LEFT_COLUMN, TOP_ROW + index, line[0], line[1], BACKGROUND)
