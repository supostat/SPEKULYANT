class_name SummaryScreen
extends RefCounted
## Итоги жизни (docs/SPEKULYANT.md §11.2, §13.3 п. 10, макет «Итоги жизни»): газетная вырезка в духе
## эпохи — некролог, приговор или заметка, — капитал, лучшая и худшая сделки, срок жизни и
## открытия для следующей. Любая клавиша ведёт к выбору новой жизни.

const TEXT := 7
const BACKGROUND := 1
const PAPER := Rect2i(4, 1, 72, 23)
const PAPER_TITLE := "ИТОГИ ЖИЗНИ"
const PAPER_FOREGROUND := 0
const PAPER_BACKGROUND := 7
const COLUMN := 8
const SOURCE_ROW := 3
const SOURCE_FOREGROUND := 8
const HEADLINE_ROW := 5
const HEADLINE_FOREGROUND := 4
const TEXT_ROW := 7
const TEXT_WIDTH := 64
const TEXT_ROWS := 6
const LINES_ROW := 14
const LABEL_WIDTH := 32
const LINE_FOREGROUND := 1
const DISCOVERY_FOREGROUND := 4
const HINT := "Нажми любую клавишу, чтобы начать новую жизнь"
const HINT_ROW := 22
const HINT_FOREGROUND := 8


func draw(view: Dictionary, screen: TextScreen) -> void:
	var summary: Dictionary = view["summary"]
	screen.clear(TEXT, BACKGROUND)
	screen.box(PAPER.position.x, PAPER.position.y, PAPER.size.x, PAPER.size.y, PAPER_FOREGROUND, PAPER_BACKGROUND, PAPER_TITLE)
	screen.put(COLUMN, SOURCE_ROW, summary["source"], SOURCE_FOREGROUND, PAPER_BACKGROUND)
	screen.put(COLUMN, HEADLINE_ROW, summary["headline"], HEADLINE_FOREGROUND, PAPER_BACKGROUND)
	var text := TextScreen.wrap(summary["text"], TEXT_WIDTH)
	for index in mini(text.size(), TEXT_ROWS):
		screen.put(COLUMN, TEXT_ROW + index, text[index], PAPER_FOREGROUND, PAPER_BACKGROUND)
	var row := LINES_ROW
	for line: Dictionary in summary["lines"]:
		screen.put(COLUMN, row, TextScreen.dot_leader(line["label"] + " ", LABEL_WIDTH) + " " + line["value"], LINE_FOREGROUND, PAPER_BACKGROUND)
		row += 1
	row += 1
	for discovery: String in summary["discoveries"]:
		if row >= HINT_ROW - 1:
			break
		screen.put(COLUMN, row, discovery, DISCOVERY_FOREGROUND, PAPER_BACKGROUND)
		row += 1
	screen.put(COLUMN, HINT_ROW, HINT, HINT_FOREGROUND, PAPER_BACKGROUND)


## Любая клавиша — к выбору новой жизни в лобби.
func handle(_action: Dictionary, _view: Dictionary) -> Dictionary:
	return {"lobby": "new_life"}


func answered(_answer: Dictionary) -> void:
	pass


func follow(_view: Dictionary) -> Dictionary:
	return {}
