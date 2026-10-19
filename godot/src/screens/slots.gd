class_name SlotsScreen
extends RefCounted
## Сохранения свободного режима (docs/SPEKULYANT.md §11.4): восемь слотов в каталоге игрока. Запись
## поверх занятого слота спрашивает подтверждения. Загрузка в лобби начинает сохранённую жизнь, в
## жизни — заменяет идущую и возвращает на экран режима.

const SAVE := "save"
const LOAD := "load"
const TITLES := {SAVE: "Сохранить жизнь", LOAD: "Загрузить жизнь"}
const VERBS := {SAVE: "сохранить", LOAD: "загрузить"}
const SLOT_COUNT := 8
const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(14, 4, 52, 15)
const COLUMN := 17
const FIRST_ROW := 6
const LINE_WIDTH := 46
const LABEL_WIDTH := 22
const SLOT_LABEL := "Слот %d "
const EMPTY := "пусто"
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const EMPTY_FOREGROUND := 8
const STATUS_ROW := 15
const WARNING_FOREGROUND := 14
const DONE_FOREGROUND := 10
const SAVED := "Жизнь записана в слот %d."
const HINT := "↑↓ слот   Enter — %s   Esc — назад"
const HINT_ROW := 17
const HINT_FOREGROUND := 8
const OVERWRITE_TITLE := "Сохранить"
const OVERWRITE_QUESTION := "Записать поверх слота %d?"
const OVERWRITE_DETAIL := "Жизнь, сохранённая в нём %s, пропадёт."
const OVERWRITE_VERB := "записать"
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

var _mode := LOAD
var _slots := ListCursor.new(SLOT_COUNT)
var _status := ""
var _status_foreground := WARNING_FOREGROUND
var _overwrite: ConfirmDialog
var _loaded := false


## `{"mode": SAVE | LOAD}`.
func enter(arguments: Dictionary) -> void:
	_mode = arguments["mode"]


func draw(view: Dictionary, screen: TextScreen) -> void:
	var saved := _saved(view)
	screen.clear(TEXT, BACKGROUND)
	screen.shadow(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, TITLES[_mode])
	for index: int in _slots.shown(SLOT_COUNT):
		var slot := _slot_id(index)
		var when: String = saved.get(slot, EMPTY)
		var line := TextScreen.pad_right(TextScreen.dot_leader(SLOT_LABEL % (index + 1), LABEL_WIDTH) + " " + when, LINE_WIDTH)
		var foreground := TEXT if saved.has(slot) else EMPTY_FOREGROUND
		var background := BACKGROUND
		if index == _slots.selected:
			foreground = SELECTED_FOREGROUND
			background = SELECTED_BACKGROUND
		screen.put(COLUMN, FIRST_ROW + index, line, foreground, background)
	screen.put(COLUMN, STATUS_ROW, TextScreen.pad_right(_status, LINE_WIDTH), _status_foreground, BACKGROUND)
	screen.put(COLUMN, HINT_ROW, HINT % VERBS[_mode], HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)
	if _overwrite != null:
		_overwrite.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if _overwrite != null:
		return _handle_overwrite(action)
	match action["action"]:
		"up":
			_slots.move(-1, SLOT_COUNT)
		"down":
			_slots.move(1, SLOT_COUNT)
		"click":
			var row := _slots.row_at(action["row"], FIRST_ROW, SLOT_COUNT)
			if row < 0:
				return {}
			_slots.select(row, SLOT_COUNT)
		"confirm":
			return _use(_slot_id(_slots.selected), _saved(view), view)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				BACK_KEY:
					return {"pop": true}
			return {}
		_:
			return {}
	_status = ""
	return {"redraw": true}


func answered(answer: Dictionary) -> void:
	if answer.has("error"):
		_status = answer["message"]
		_status_foreground = WARNING_FOREGROUND
		return
	if _mode == LOAD:
		_loaded = true
	else:
		_status = SAVED % (_slots.selected + 1)
		_status_foreground = DONE_FOREGROUND


## Загруженная жизнь идёт с экрана режима.
func follow(_view: Dictionary) -> Dictionary:
	if _loaded:
		_loaded = false
		return {"home": true}
	return {}


func _use(slot: String, saved: Dictionary, view: Dictionary) -> Dictionary:
	if _mode == SAVE:
		if saved.has(slot):
			var detail := OVERWRITE_DETAIL % saved[slot]
			_overwrite = ConfirmDialog.new(OVERWRITE_TITLE, OVERWRITE_QUESTION % (_slots.selected + 1), detail, OVERWRITE_VERB)
			return {"redraw": true}
		return {"command": {"command": "save", "slot": slot}}
	if not saved.has(slot):
		return {}
	if Router.in_life(view):
		return {"command": {"command": "load", "slot": slot}}
	return {"resume": slot}


func _handle_overwrite(action: Dictionary) -> Dictionary:
	match _overwrite.handle(action):
		ConfirmDialog.CONFIRMED:
			_overwrite = null
			return {"command": {"command": "save", "slot": _slot_id(_slots.selected)}}
		ConfirmDialog.CANCELLED:
			_overwrite = null
			return {"redraw": true}
	return {}


## Слоты свободного режима — «1»…«8»; слот ironman-жизни сюда не попадает.
func _slot_id(index: int) -> String:
	return str(index + 1)


## Записанные слоты: слот → когда записан.
func _saved(view: Dictionary) -> Dictionary:
	var saved := {}
	for slot: Dictionary in view["slots"]:
		saved[slot["slot"]] = slot["saved"]
	return saved
