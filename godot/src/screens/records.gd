class_name RecordsScreen
extends RefCounted
## Рекорды (docs/SPEKULYANT.md §11.4, решение 15.10): «Рейтинг Форбс» ironman-жизней и таблица
## свободного режима — имя и капитал в рублях 1984 года, под ними биография, годы жизни и финал.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(2, 1, 76, 22)
const TITLE := "Рекорды"
const COLUMN := 4
const FIRST_ROW := 3
const VISIBLE := 16
const WIDTH := 72
const TABLES := [
	["ironman", "Рейтинг Форбс — только ironman-жизни"],
	["free", "Свободный режим"],
]
const TABLE_FOREGROUND := 14
const COLUMNS_LINE_FOREGROUND := 8
const CAPITAL_WIDTH := 16
const COLUMN_TITLES := ["Имя", "Капитал"]
## Запись — две строки: имя целиком и капитал, под ними биография, годы и финал.
const DETAILS_INDENT := "  "
const DETAILS_SEPARATOR := " · "
const DETAILS_FOREGROUND := 8
const EMPTY := "Пока пусто."
const EMPTY_FOREGROUND := 8
const HINT := "↑↓ PgUp PgDn — листать   Esc — назад"
const HINT_ROW := 21
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
const HELP_KEY := 1
const BACK_KEY := 10

var _text := ScrollText.new(VISIBLE)


func draw(view: Dictionary, screen: TextScreen) -> void:
	screen.clear(TEXT, BACKGROUND)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, TITLE)
	_text.draw(_lines(view["records"]), screen, COLUMN, FIRST_ROW, WIDTH, BACKGROUND)
	screen.put(COLUMN, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	match action["action"]:
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
			return {}
	return {"redraw": true} if _text.handle(action, _lines(view["records"]).size()) else {}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _lines(records: Dictionary) -> Array:
	var lines: Array = []
	for table: Array in TABLES:
		if not lines.is_empty():
			lines.append(["", TEXT])
		lines.append([table[1], TABLE_FOREGROUND])
		var rows: Array = records[table[0]]
		if rows.is_empty():
			lines.append([EMPTY, EMPTY_FOREGROUND])
			continue
		lines.append([_row(COLUMN_TITLES[0], COLUMN_TITLES[1]), COLUMNS_LINE_FOREGROUND])
		for record: Dictionary in rows:
			lines.append([_row(record["name"], record["capital"]), TEXT])
			var details := DETAILS_SEPARATOR.join(PackedStringArray([record["biography"], record["years"], record["ending"]]))
			lines.append([DETAILS_INDENT + details, DETAILS_FOREGROUND])
	return lines


func _row(name: String, capital: String) -> String:
	return TextScreen.pad_right(name, WIDTH - CAPITAL_WIDTH) + TextScreen.pad_left(capital, CAPITAL_WIDTH)
