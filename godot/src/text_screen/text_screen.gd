class_name TextScreen
extends Control
## Текстовый режим 80×25: буфер ячеек «символ, цвет символа, цвет фона» и отрисовка глифами VGA 9×16.
## Примитивы повторяют класс Scr дизайн-книги: put, fill, box, shadow, fkeys.

const COLUMNS := 80
const ROWS := 25
const CELL_SIZE := Vector2(9, 16)
const ASPECT_CORRECTION := Vector2(1.0, 1.35)
const FONT_SIZE := 16
## Сглаживание, хинтинг и субпиксельное позиционирование выключены в параметрах импорта шрифта.
const FONT: FontFile = preload("res://assets/fonts/PxPlus_IBM_VGA_9x16.ttf")
const INITIAL_FOREGROUND := 7
const INITIAL_BACKGROUND := 1
const SINGLE_FRAME: Array[String] = ["┌", "┐", "└", "┘", "─", "│"]
const DOUBLE_FRAME: Array[String] = ["╔", "╗", "╚", "╝", "═", "║"]
const SHADOW_FOREGROUND := 8
const SHADOW_BACKGROUND := 0
const FKEY_ROW := 24
const FKEY_SLOT_WIDTH := 9
const FKEY_KEY_FOREGROUND := 15
const FKEY_KEY_BACKGROUND := 0
const FKEY_LABEL_FOREGROUND := 0
const FKEY_LABEL_BACKGROUND := 3

var _characters := PackedStringArray()
var _foreground := PackedByteArray()
var _background := PackedByteArray()
## Номер F-клавиши в каждой колонке строки F-клавиш; 0 — колонка без клавиши или без подписи.
var _fkey_columns := PackedInt32Array()


func _init() -> void:
	_characters.resize(COLUMNS * ROWS)
	_foreground.resize(COLUMNS * ROWS)
	_background.resize(COLUMNS * ROWS)
	_fkey_columns.resize(COLUMNS)
	clear(INITIAL_FOREGROUND, INITIAL_BACKGROUND)
	size = CELL_SIZE * Vector2(COLUMNS, ROWS)
	scale = ASPECT_CORRECTION
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	mouse_filter = Control.MOUSE_FILTER_IGNORE


func clear(foreground: int, background: int) -> void:
	fill(0, 0, COLUMNS, ROWS, foreground, background)


func put(column: int, row: int, text: String, foreground: int, background: int) -> void:
	for offset in text.length():
		var cell_column := column + offset
		if _is_inside(cell_column, row):
			var index := row * COLUMNS + cell_column
			_characters[index] = text[offset]
			_foreground[index] = foreground
			_background[index] = background
	queue_redraw()


func fill(column: int, row: int, width: int, height: int, foreground: int, background: int) -> void:
	for line in height:
		put(column, row + line, " ".repeat(width), foreground, background)


func box(column: int, row: int, width: int, height: int, foreground: int, background: int, title := "", is_double := true) -> void:
	var frame := DOUBLE_FRAME if is_double else SINGLE_FRAME
	fill(column, row, width, height, foreground, background)
	put(column, row, frame[0] + frame[4].repeat(width - 2) + frame[1], foreground, background)
	for line in range(1, height - 1):
		put(column, row + line, frame[5], foreground, background)
		put(column + width - 1, row + line, frame[5], foreground, background)
	put(column, row + height - 1, frame[2] + frame[4].repeat(width - 2) + frame[3], foreground, background)
	if not title.is_empty():
		var caption := " " + title + " "
		put(column + floori((width - caption.length()) / 2.0), row, caption, foreground, background)


func shadow(column: int, row: int, width: int, height: int) -> void:
	for line in range(1, height + 1):
		_shade(column + width, row + line)
		_shade(column + width + 1, row + line)
	for offset in range(2, width + 2):
		_shade(column + offset, row + height)
	queue_redraw()


## Строка F-клавиш: номер клавиши белым на чёрном, подпись чёрным на бирюзовом, слот в девять колонок.
## Слоты запоминаются: по ним мышь находит клавишу.
func fkeys(keys: Array) -> void:
	_fkey_columns.fill(0)
	var column := 0
	for key_and_label: Array in keys:
		var key: String = key_and_label[0]
		var label: String = key_and_label[1]
		put(column, FKEY_ROW, key, FKEY_KEY_FOREGROUND, FKEY_KEY_BACKGROUND)
		var label_width := FKEY_SLOT_WIDTH - key.length()
		put(column + key.length(), FKEY_ROW, pad_right(label, label_width), FKEY_LABEL_FOREGROUND, FKEY_LABEL_BACKGROUND)
		if not label.is_empty():
			for slot_column in range(column, mini(column + FKEY_SLOT_WIDTH, COLUMNS)):
				_fkey_columns[slot_column] = int(key)
		column += FKEY_SLOT_WIDTH
	if column < COLUMNS:
		put(column, FKEY_ROW, " ".repeat(COLUMNS - column), FKEY_LABEL_FOREGROUND, FKEY_LABEL_BACKGROUND)


## Номер F-клавиши под колонкой строки F-клавиш; 0 — там клавиши нет.
func fkey_at(column: int) -> int:
	return _fkey_columns[column] if column >= 0 and column < COLUMNS else 0


## Клетка экрана под точкой окна; за пределами экрана — (-1, -1).
func cell_at(point: Vector2) -> Vector2i:
	var local := get_global_transform_with_canvas().affine_inverse() * point
	var cell := Vector2i(floori(local.x / CELL_SIZE.x), floori(local.y / CELL_SIZE.y))
	return cell if _is_inside(cell.x, cell.y) else Vector2i(-1, -1)


## Текст строки экрана — по нему инструменты проверяют, что показано.
func row_text(row: int) -> String:
	return "".join(_characters.slice(row * COLUMNS, (row + 1) * COLUMNS))


## Буфер в формате .screen: 25 строк символов, пустая строка, 25 строк hex-цветов символов, пустая строка, 25 строк hex-цветов фона.
func dump() -> String:
	var lines := PackedStringArray()
	for row in ROWS:
		lines.append(row_text(row))
	lines.append("")
	for row in ROWS:
		lines.append(_hex_row(_foreground, row))
	lines.append("")
	for row in ROWS:
		lines.append(_hex_row(_background, row))
	return "\n".join(lines) + "\n"


func _draw() -> void:
	var baseline := Vector2(0, FONT.get_ascent(FONT_SIZE))
	for row in ROWS:
		for column in COLUMNS:
			var index := row * COLUMNS + column
			var origin := Vector2(column, row) * CELL_SIZE
			draw_rect(Rect2(origin, CELL_SIZE), Ega.COLORS[_background[index]])
			if _characters[index] != " ":
				draw_char(FONT, origin + baseline, _characters[index], FONT_SIZE, Ega.COLORS[_foreground[index]])


func _shade(column: int, row: int) -> void:
	if _is_inside(column, row):
		var index := row * COLUMNS + column
		_foreground[index] = SHADOW_FOREGROUND
		_background[index] = SHADOW_BACKGROUND


func _is_inside(column: int, row: int) -> bool:
	return column >= 0 and column < COLUMNS and row >= 0 and row < ROWS


func _hex_row(colors: PackedByteArray, row: int) -> String:
	var digits := PackedStringArray()
	for column in COLUMNS:
		digits.append("%x" % colors[row * COLUMNS + column])
	return "".join(digits)


## Дополняет пробелами справа или обрезает до ширины — колонка, выровненная влево.
static func pad_right(text: String, width: int) -> String:
	return text.substr(0, width) if text.length() >= width else text + " ".repeat(width - text.length())


## Дополняет пробелами слева — колонка, выровненная вправо; длинный текст не обрезается.
static func pad_left(text: String, width: int) -> String:
	return text if text.length() >= width else " ".repeat(width - text.length()) + text


## Переносит текст по словам в строки не длиннее ширины.
static func wrap(text: String, width: int) -> PackedStringArray:
	var lines := PackedStringArray()
	var line := ""
	for word in text.split(" "):
		if (line + " " + word).strip_edges().length() > width:
			lines.append(line)
			line = word
		else:
			line = word if line.is_empty() else line + " " + word
	if not line.is_empty():
		lines.append(line)
	return lines
