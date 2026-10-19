class_name Saves
extends RefCounted
## Каталог игрока (§11.4): жизни — `saves/<слот>.spk`, метапрогрессия — `meta.spk`, по умолчанию в
## `user://`. Пишет и читает файлы ядро через мост — атомарно, с проверкой имени слота и версий
## формата; здесь каталог переводится в путь ОС для старта жизни и собираются слоты для меню.

const PLAYER_ROOT := "user://"
const SAVES_FOLDER := "saves"
const META_FILE := "meta.spk"
const EXTENSION := "spk"

var _root: String


func _init(root: String = PLAYER_ROOT) -> void:
	_root = root


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


func _directory() -> String:
	return ProjectSettings.globalize_path(_root)
