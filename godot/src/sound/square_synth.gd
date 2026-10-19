class_name SquareSynth
extends RefCounted
## Одноголосая прямоугольная волна пищалки PC-speaker. Тоны — {"hertz": частота или null — пауза,
## "milliseconds": длительность} — звучат по очереди; конец каждого тона молчит коротким зазором,
## чтобы повторы одной ноты звучали раздельно. Новый напев сменяет недоигранный: голос у пищалки
## один.

const AMPLITUDE := 0.2
const GAP_MILLISECONDS := 12.0
const MILLISECONDS_IN_SECOND := 1000.0
const HALF_PERIOD := 0.5

var _mix_rate: float
var _tones: Array = []
var _hertz := 0.0
var _phase := 0.0
## Сколько кадров текущего тона ещё звучит и сколько молчит после.
var _sounding := 0
var _silent := 0


func _init(mix_rate: float) -> void:
	_mix_rate = mix_rate


func play(tones: Array) -> void:
	_tones = tones.duplicate()
	_sounding = 0
	_silent = 0


func is_playing() -> bool:
	return _sounding > 0 or _silent > 0 or not _tones.is_empty()


## Следующие `frame_count` кадров стерео; когда напев кончился, остаток — тишина.
func fill(frame_count: int) -> PackedVector2Array:
	var frames := PackedVector2Array()
	frames.resize(frame_count)
	for index in frame_count:
		if _sounding == 0 and _silent == 0:
			if _tones.is_empty():
				break
			_next_tone()
		if _sounding > 0:
			_phase = fmod(_phase + _hertz / _mix_rate, 1.0)
			var level := AMPLITUDE if _phase < HALF_PERIOD else -AMPLITUDE
			frames[index] = Vector2(level, level)
			_sounding -= 1
		elif _silent > 0:
			_silent -= 1
	return frames


func _next_tone() -> void:
	var tone: Dictionary = _tones.pop_front()
	var frames := _frames_of(float(tone["milliseconds"]))
	if tone["hertz"] == null:
		_sounding = 0
		_silent = frames
		return
	var gap := mini(_frames_of(GAP_MILLISECONDS), frames)
	_hertz = float(tone["hertz"])
	_sounding = frames - gap
	_silent = gap


func _frames_of(milliseconds: float) -> int:
	return roundi(milliseconds * _mix_rate / MILLISECONDS_IN_SECOND)
