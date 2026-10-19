class_name Header
extends RefCounted
## Шапка экранов на воле (макеты «Рынок» и «Событие»): название игры и строка «город │ дата │ деньги».

const TITLE := " СПЕКУЛЯНТЪ "
const TITLE_FOREGROUND := 15
const TITLE_BACKGROUND := 4
const LINE_COLUMN := 12
const LINE_WIDTH := 68
const LINE_FOREGROUND := 0
const LINE_BACKGROUND := 3
const SEPARATOR := " │ "


static func draw(header: Dictionary, screen: TextScreen) -> void:
	screen.put(0, 0, TITLE, TITLE_FOREGROUND, TITLE_BACKGROUND)
	var parts := PackedStringArray([header["city"], header["date"]])
	parts.append_array(PackedStringArray(header["money"]))
	var line := " " + SEPARATOR.join(parts) + " "
	screen.put(LINE_COLUMN, 0, TextScreen.pad_right(line, LINE_WIDTH), LINE_FOREGROUND, LINE_BACKGROUND)
