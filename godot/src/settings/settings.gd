class_name Settings
extends RefCounted
## Настройки игрока в `user://settings.cfg`: CRT-эффект — по умолчанию выключен, — пищалка
## PC-speaker — по умолчанию включена — и полный экран — по умолчанию окно. Это вкус игрока, а не
## мир игры, поэтому они живут в Godot, а не в ядре; экраны видят их в снимке под ключом `settings`.

signal changed

const PLAYER_FILE := "user://settings.cfg"
const SECTION := "player"
const CRT := "crt"
const SOUND := "sound"
const FULLSCREEN := "fullscreen"
const DEFAULTS := {CRT: false, SOUND: true, FULLSCREEN: false}

var _path: String
var _config := ConfigFile.new()


## Файла ещё нет — настройки по умолчанию.
func _init(path: String = PLAYER_FILE) -> void:
	_path = path
	if FileAccess.file_exists(path):
		var loaded := _config.load(path)
		if loaded != OK:
			push_error("настройки %s не читаются: %s — остаются по умолчанию" % [path, error_string(loaded)])


func enabled(key: String) -> bool:
	return _config.get_value(SECTION, key, DEFAULTS[key])


func toggle(key: String) -> void:
	_config.set_value(SECTION, key, not enabled(key))
	var saved := _config.save(_path)
	if saved != OK:
		push_error("настройки не записаны в %s: %s" % [_path, error_string(saved)])
	changed.emit()


## Кладёт в снимок ядра настройки для экрана настроек — `settings`: ключ → включено ли.
func annotate(view: Dictionary) -> void:
	var values := {}
	for key: String in DEFAULTS:
		values[key] = enabled(key)
	view["settings"] = values
