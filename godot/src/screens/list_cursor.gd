class_name ListCursor
extends RefCounted
## Выделенная строка списка и окно прокрутки: выделение всегда видно, список листается по строке.

var selected := 0
var first := 0
var _visible: int


func _init(visible: int) -> void:
	_visible = visible


## Выделяет строку `row` из `count`; окно прокрутки идёт за выделением.
func select(row: int, count: int) -> void:
	selected = clampi(row, 0, maxi(count - 1, 0))
	first = clampi(first, 0, maxi(count - _visible, 0))
	if selected < first:
		first = selected
	elif selected >= first + _visible:
		first = selected - _visible + 1


func move(step: int, count: int) -> void:
	select(selected + step, count)


## Номера строк списка, видимые в окне.
func shown(count: int) -> Array:
	select(selected, count)
	return range(first, mini(count, first + _visible))


## Строка списка под строкой экрана `screen_row`, если окно начинается со строки `top`; −1 — мимо.
func row_at(screen_row: int, top: int, count: int) -> int:
	var row := first + screen_row - top
	if screen_row < top or screen_row >= top + _visible or row >= count:
		return -1
	return row
