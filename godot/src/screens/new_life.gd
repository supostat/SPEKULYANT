class_name NewLifeScreen
extends RefCounted
## Новая жизнь (docs/SPEKULYANT.md §11.1, §11.4, §13.3 п. 11): биография — стартовая или открытая
## прошлыми жизнями, режим ironman или свободный и сид мира. Закрытая биография показывает, чем её
## открыть, а начать её не даст ядро. Стрелки выбирают биографию, ◄ ► и F2 меняют режим, цифры
## набирают сид, F3 — новый случайный, Enter — жить. Новая ironman-жизнь занимает слот прежней
## незаконченной — это спрашивается.

const TEXT := 7
const BACKGROUND := 1
const FRAME := 15
const BOX := Rect2i(1, 1, 78, 22)
const TITLE := "Новая жизнь"
const HEADING := "Биография"
const HEADING_FOREGROUND := 14
const LIST_COLUMN := 3
const HEADING_ROW := 3
const LIST_ROW := 5
const LIST_WIDTH := 30
const LIST_VISIBLE := 12
const SELECTED_FOREGROUND := 0
const SELECTED_BACKGROUND := 3
const LOCKED_FOREGROUND := 8
const DETAIL_COLUMN := 36
const DETAIL_WIDTH := 40
const DETAIL_TITLE_FOREGROUND := 15
const DETAIL_ROWS := 5
const LOCKED_PREFIX := "Закрыта: "
const LOCKED_HINT_FOREGROUND := 14
const LABEL_WIDTH := 12
const MODE_ROW := 13
const MODE_LABEL := "Режим"
const MODE_VALUE_WIDTH := 10
const MODE_FOREGROUND := 15
const NOTE_FOREGROUND := 8
const MODES := [
	{"ironman": true, "label": "ironman", "note": "одна жизнь без отката, «Рейтинг Форбс»"},
	{"ironman": false, "label": "свободный", "note": "сохранения в любой момент, своя таблица"},
]
const SEED_ROW := 16
const SEED_LABEL := "Сид мира"
const SEED_NOTE := "цифры — свой сид, F3 — случайный"
const SEED_FIELD_FOREGROUND := 0
const SEED_FIELD_BACKGROUND := 3
const LONGEST_SEED := 9
const RANDOM_SEEDS := 1000000
const WARNING_ROW := 19
const WARNING_FOREGROUND := 14
const HINT := "↑↓ биография   ◄► режим   Enter — жить   Esc — назад"
const HINT_ROW := 21
const HINT_FOREGROUND := 8
const FKEYS := [
	["1", "Помощь"],
	["2", "Режим"],
	["3", "Сид"],
	["4", ""],
	["5", ""],
	["6", ""],
	["7", ""],
	["10", "Назад"],
]
const HELP_KEY := 1
const MODE_KEY := 2
const SEED_KEY := 3
const BACK_KEY := 10
const ABANDON_TITLE := "Новая жизнь"
const ABANDON_QUESTION := "Бросить незаконченную ironman-жизнь?"
const ABANDON_DETAIL := "Её слот займёт новая: ironman не откатывается."
const ABANDON_VERB := "начать заново"

var _biographies := ListCursor.new(LIST_VISIBLE)
var _mode := 0
var _seed := str(randi() % RANDOM_SEEDS)
var _seed_typed := false
var _warning := ""
var _abandon: ConfirmDialog


func draw(view: Dictionary, screen: TextScreen) -> void:
	var biographies: Array = view["biographies"]
	screen.clear(TEXT, BACKGROUND)
	screen.box(BOX.position.x, BOX.position.y, BOX.size.x, BOX.size.y, FRAME, BACKGROUND, TITLE)
	screen.put(LIST_COLUMN, HEADING_ROW, HEADING, HEADING_FOREGROUND, BACKGROUND)
	for index: int in _biographies.shown(biographies.size()):
		var biography: Dictionary = biographies[index]
		var foreground := TEXT if biography["unlocked"] else LOCKED_FOREGROUND
		var background := BACKGROUND
		if index == _biographies.selected:
			foreground = SELECTED_FOREGROUND
			background = SELECTED_BACKGROUND
		var line := TextScreen.pad_right(" " + biography["title"], LIST_WIDTH)
		screen.put(LIST_COLUMN, LIST_ROW + index - _biographies.first, line, foreground, background)
	if not biographies.is_empty():
		_draw_biography(biographies[_biographies.selected], screen)
	var mode: Dictionary = MODES[_mode]
	screen.put(DETAIL_COLUMN, MODE_ROW, TextScreen.dot_leader(MODE_LABEL, LABEL_WIDTH), TEXT, BACKGROUND)
	screen.put(DETAIL_COLUMN + LABEL_WIDTH + 1, MODE_ROW, "◄ " + TextScreen.pad_right(mode["label"], MODE_VALUE_WIDTH) + " ►", MODE_FOREGROUND, BACKGROUND)
	screen.put(DETAIL_COLUMN, MODE_ROW + 1, mode["note"], NOTE_FOREGROUND, BACKGROUND)
	screen.put(DETAIL_COLUMN, SEED_ROW, TextScreen.dot_leader(SEED_LABEL, LABEL_WIDTH), TEXT, BACKGROUND)
	screen.put(DETAIL_COLUMN + LABEL_WIDTH + 1, SEED_ROW, TextScreen.pad_left(_seed, LONGEST_SEED + 1), SEED_FIELD_FOREGROUND, SEED_FIELD_BACKGROUND)
	screen.put(DETAIL_COLUMN, SEED_ROW + 1, SEED_NOTE, NOTE_FOREGROUND, BACKGROUND)
	screen.put(LIST_COLUMN, WARNING_ROW, TextScreen.pad_right(_warning, BOX.size.x - 4), WARNING_FOREGROUND, BACKGROUND)
	screen.put(LIST_COLUMN, HINT_ROW, HINT, HINT_FOREGROUND, BACKGROUND)
	screen.fkeys(FKEYS)
	if _abandon != null:
		_abandon.draw(screen)


func handle(action: Dictionary, view: Dictionary) -> Dictionary:
	if _abandon != null:
		return _handle_abandon(action, view)
	var biographies: Array = view["biographies"]
	match action["action"]:
		"up":
			_biographies.move(-1, biographies.size())
		"down":
			_biographies.move(1, biographies.size())
		"left", "right":
			_mode = 1 - _mode
		"digit":
			var typed: String = (_seed if _seed_typed else "") + str(action["digit"])
			_seed = typed.substr(0, LONGEST_SEED).lstrip("0")
			_seed_typed = true
		"erase":
			_seed = _seed.substr(0, maxi(_seed.length() - 1, 0))
			_seed_typed = true
		"click":
			var row := _biographies.row_at(action["row"], LIST_ROW, biographies.size())
			if row >= 0:
				_biographies.select(row, biographies.size())
			elif action["row"] == MODE_ROW:
				_mode = 1 - _mode
			else:
				return {}
		"confirm":
			if biographies.is_empty():
				return {}
			if MODES[_mode]["ironman"] and Saves.holds_ironman(view):
				_abandon = ConfirmDialog.new(ABANDON_TITLE, ABANDON_QUESTION, ABANDON_DETAIL, ABANDON_VERB)
				return {"redraw": true}
			return _start(biographies)
		"back":
			return {"pop": true}
		"fkey":
			match action["key"]:
				HELP_KEY:
					return {"push": "help"}
				MODE_KEY:
					_mode = 1 - _mode
				SEED_KEY:
					_seed = str(randi() % RANDOM_SEEDS)
					_seed_typed = false
				BACK_KEY:
					return {"pop": true}
				_:
					return {}
		_:
			return {}
	_warning = ""
	return {"redraw": true}


## Ядро не начало жизнь — закрытая биография или нечитаемый каталог: отказ остаётся на экране.
func answered(answer: Dictionary) -> void:
	_abandon = null
	_warning = answer["message"] if answer.has("error") else ""


func follow(_view: Dictionary) -> Dictionary:
	return {}


func _start(biographies: Array) -> Dictionary:
	var life := {
		"biography": biographies[_biographies.selected]["biography"],
		"ironman": MODES[_mode]["ironman"],
		"seed": int(_seed),
	}
	return {"new_life": life}


func _handle_abandon(action: Dictionary, view: Dictionary) -> Dictionary:
	match _abandon.handle(action):
		ConfirmDialog.CONFIRMED:
			return _start(view["biographies"])
		ConfirmDialog.CANCELLED:
			_abandon = null
			return {"redraw": true}
	return {}


## Название, с чего начинается жизнь, и для закрытой — чем её открыть; всё над строкой режима.
func _draw_biography(biography: Dictionary, screen: TextScreen) -> void:
	screen.put(DETAIL_COLUMN, LIST_ROW, biography["title"], DETAIL_TITLE_FOREGROUND, BACKGROUND)
	var lines: Array = []
	for line in TextScreen.wrap(biography["start"], DETAIL_WIDTH):
		lines.append([line, TEXT])
	if biography["unlock_hint"] != null:
		for line in TextScreen.wrap(LOCKED_PREFIX + biography["unlock_hint"], DETAIL_WIDTH):
			lines.append([line, LOCKED_HINT_FOREGROUND])
	for index in mini(lines.size(), DETAIL_ROWS):
		screen.put(DETAIL_COLUMN, LIST_ROW + 2 + index, lines[index][0], lines[index][1], BACKGROUND)
