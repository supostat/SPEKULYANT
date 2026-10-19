class_name ScrollText
extends RefCounted
## Текст окна строками со своим цветом — `[текст, цвет]`: ↑↓ листают по строке, PgUp и PgDn — по
## окну; внизу окна — какие строки видны.

const POSITION_FOREGROUND := 8
const POSITION := "строки %d–%d из %d"

var first := 0
var _visible: int


func _init(visible: int) -> void:
	_visible = visible


func draw(lines: Array, screen: TextScreen, column: int, row: int, width: int, background: int) -> void:
	first = clampi(first, 0, maxi(lines.size() - _visible, 0))
	for index in range(first, mini(lines.size(), first + _visible)):
		var line: Array = lines[index]
		screen.put(column, row + index - first, TextScreen.pad_right(line[0], width), line[1], background)
	if lines.size() > _visible:
		var position := POSITION % [first + 1, mini(first + _visible, lines.size()), lines.size()]
		screen.put(column + width - position.length(), row + _visible, position, POSITION_FOREGROUND, background)


## `true`, если текст сдвинулся.
func handle(action: Dictionary, count: int) -> bool:
	var before := first
	match action["action"]:
		"up":
			first -= 1
		"down":
			first += 1
		"page_up":
			first -= _visible
		"page_down":
			first += _visible
	first = clampi(first, 0, maxi(count - _visible, 0))
	return first != before
