class_name TimeIndicator
extends RefCounted
## Ход времени под окнами (макет «Рынок»): пауза или скорость стрелками и подсказка клавиш.

const COLUMN := 1
const ROW := 23
const BACKGROUND := 1
const PAUSED_LINE := " ПАУЗА  (пробел)"
const PAUSED_FOREGROUND := 14
const RUNNING_PREFIX := " Время идёт  "
const RUNNING_SPEED_MARK := "►"
const RUNNING_SUFFIX := "  (+/−, пробел)"
const RUNNING_FOREGROUND := 10


static func draw(time: Dictionary, screen: TextScreen) -> void:
	if time["paused"]:
		screen.put(COLUMN, ROW, PAUSED_LINE, PAUSED_FOREGROUND, BACKGROUND)
		return
	var line := RUNNING_PREFIX + RUNNING_SPEED_MARK.repeat(int(time["speed"])) + RUNNING_SUFFIX
	screen.put(COLUMN, ROW, line, RUNNING_FOREGROUND, BACKGROUND)
