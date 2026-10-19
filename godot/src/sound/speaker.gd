class_name Speaker
extends AudioStreamPlayer
## Пищалка PC-speaker (docs/SPEKULYANT.md §13.1 п. 5, §13.2): прямоугольная волна SquareSynth через
## AudioStreamGenerator — мелодия заставки и бипы интерфейса: событие остановило время, пришло
## сообщение на пейджер, ядро отказало. Выключенная в настройках пищалка молчит.

const MIX_RATE := 22050.0
const BUFFER_SECONDS := 0.1
const EVENT := "event"
const PAGER := "pager"
const ERROR := "error"
const CUES := {
	EVENT: [
		{"hertz": 880, "milliseconds": 90},
		{"hertz": null, "milliseconds": 30},
		{"hertz": 660, "milliseconds": 150},
	],
	PAGER: [
		{"hertz": 2000, "milliseconds": 60},
		{"hertz": null, "milliseconds": 50},
		{"hertz": 2000, "milliseconds": 60},
		{"hertz": null, "milliseconds": 50},
		{"hertz": 2000, "milliseconds": 60},
	],
	ERROR: [
		{"hertz": 196, "milliseconds": 250},
	],
}

var _synth := SquareSynth.new(MIX_RATE)
var _enabled := true


func _ready() -> void:
	var generator := AudioStreamGenerator.new()
	generator.mix_rate = MIX_RATE
	generator.buffer_length = BUFFER_SECONDS
	stream = generator
	play()


## Напев — мелодия заставки; у выключенной пищалки он не звучит.
func play_tones(tones: Array) -> void:
	if _enabled:
		_synth.play(tones)


## Бип интерфейса по имени: EVENT, PAGER или ERROR.
func cue(cue_name: String) -> void:
	play_tones(CUES[cue_name])


## Выключенная пищалка обрывает недоигранное и молчит, пока её не включат.
func set_enabled(enabled: bool) -> void:
	_enabled = enabled
	if not enabled:
		_synth.play([])


func is_sounding() -> bool:
	return _synth.is_playing()


## Пока поток играет, генератор получает столько кадров, сколько в нём свободно.
func _process(_delta: float) -> void:
	if not playing:
		return
	var playback := get_stream_playback() as AudioStreamGeneratorPlayback
	var available := playback.get_frames_available()
	if available > 0:
		playback.push_buffer(_synth.fill(available))
