class_name Saves
extends RefCounted
## Каталог игрока (§11.4): жизни — `saves/<слот>.spk`, метапрогрессия — `meta.spk`, по умолчанию в
## `user://`. Пишет и читает файлы ядро через мост — атомарно, с проверкой имени слота и версий
## формата; здесь каталог переводится в путь ОС для старта сеанса и собираются слоты для меню.

const PLAYER_ROOT := "user://"
const SAVES_FOLDER := "saves"
const META_FILE := "meta.spk"
const EXTENSION := "spk"
## Слот ironman-жизни: она пишется в него сама, а меню продолжает её отсюда.
const IRONMAN_SLOT := "ironman"
const SECONDS_IN_MINUTE := 60

var _root: String


func _init(root: String = PLAYER_ROOT) -> void:
	_root = root


## Старт лобби для `SpeculantGame.new_life`: каталог открыт, жизнь не начата.
func lobby() -> String:
	return JSON.stringify({"storage": _directory()})


## Старт новой жизни для `SpeculantGame.new_life`.
func new_life(biography: String, ironman: bool, seed: int) -> String:
	var life := {"biography": biography, "ironman": ironman, "seed": seed}
	return JSON.stringify({"storage": _directory(), "life": life})


## Старт жизни, сохранённой в слоте.
func resume(slot: String) -> String:
	return JSON.stringify({"storage": _directory(), "resume": slot})


## Слоты с сохранёнными жизнями по алфавиту.
func slots() -> PackedStringArray:
	var found := PackedStringArray()
	var folder := _root.path_join(SAVES_FOLDER)
	if not DirAccess.dir_exists_absolute(folder):
		return found
	for file in DirAccess.get_files_at(folder):
		if file.get_extension() == EXTENSION:
			found.append(file.get_basename())
	found.sort()
	return found


## Хотя бы одна жизнь уже кончилась и оставила знания следующим.
func has_past_lives() -> bool:
	return FileAccess.file_exists(_root.path_join(META_FILE))


## Кладёт в снимок ядра слоты каталога для меню — `slots`: слот и когда он записан по часам игрока.
func annotate(view: Dictionary) -> void:
	var listed: Array = []
	for slot in slots():
		listed.append({"slot": slot, "saved": _saved_at(slot)})
	view["slots"] = listed


## В снимке со слотами каталога лежит незаконченная ironman-жизнь.
static func holds_ironman(view: Dictionary) -> bool:
	return view["slots"].any(func(saved: Dictionary) -> bool: return saved["slot"] == IRONMAN_SLOT)


## «19.10.2026 17:40» — время записи файла слота в часовом поясе игрока.
func _saved_at(slot: String) -> String:
	var path := _root.path_join(SAVES_FOLDER).path_join(slot + "." + EXTENSION)
	var bias: int = Time.get_time_zone_from_system()["bias"]
	var moment := Time.get_datetime_dict_from_unix_time(FileAccess.get_modified_time(path) + bias * SECONDS_IN_MINUTE)
	return "%02d.%02d.%d %02d:%02d" % [moment["day"], moment["month"], moment["year"], moment["hour"], moment["minute"]]


func _directory() -> String:
	return ProjectSettings.globalize_path(_root)
