extends SceneTree
## Безголовая проверка пищалки для `cargo xtask sound-smoke`: синтезатор выдаёт прямоугольную волну
## заданной частоты — частота считается по фронтам в заполненном буфере, уровни только ±амплитуда,
## — пауза звучит тишиной, мелодия заставки из вшитых данных приходит через мост в снимке лобби и
## звучит своей первой нотой, а узел Speaker в дереве заполняет буфер AudioStreamGenerator. Мелодия
## и бипы пишутся в WAV — путь первым аргументом после `--`, — чтобы человек их послушал.

const HERTZ := [196, 440, 659, 2000]
const TONE_MILLISECONDS := 500
## Частоту меряем по началу тона, до зазора в его конце.
const MEASURED_SECONDS := 0.4
const TOLERANCE := 0.02
const SAVES_ROOT := "user://sound_smoke"
const FRAMES_TO_FILL := 3
## Остановленный поток аудиосервер отпускает на своём такте — иначе он утечёт при выходе.
const RELEASE_FRAMES := 30
const LISTENING_PAUSE := {"hertz": null, "milliseconds": 600}
const SAMPLE_SCALE := 32767.0
const OUTPUT_ARGUMENT := 0

var _speaker: Speaker


func _initialize() -> void:
	if _square_waves() and _rest_is_silent() and _boot_melody():
		_check_speaker()


func _square_waves() -> bool:
	for hertz: int in HERTZ:
		var synth := SquareSynth.new(Speaker.MIX_RATE)
		synth.play([{"hertz": hertz, "milliseconds": TONE_MILLISECONDS}])
		var frames := synth.fill(roundi(MEASURED_SECONDS * Speaker.MIX_RATE))
		var measured := _rising_edges(frames) / MEASURED_SECONDS
		if absf(measured - hertz) > hertz * TOLERANCE:
			return _fail("тон %d Гц звучит на %.1f Гц" % [hertz, measured])
		for frame in frames:
			if not is_equal_approx(absf(frame.x), SquareSynth.AMPLITUDE) or frame.x != frame.y:
				return _fail("тон %d Гц — не прямоугольная волна: кадр %s" % [hertz, frame])
	return true


func _rest_is_silent() -> bool:
	var synth := SquareSynth.new(Speaker.MIX_RATE)
	synth.play([{"hertz": null, "milliseconds": TONE_MILLISECONDS}])
	for frame in synth.fill(roundi(MEASURED_SECONDS * Speaker.MIX_RATE)):
		if frame != Vector2.ZERO:
			return _fail("пауза звучит: кадр %s" % frame)
	if not synth.is_playing():
		return _fail("пауза кончилась раньше своей длины")
	return true


## Лобби пустого каталога отдаёт мелодию заставки; первая нота звучит своей частотой.
func _boot_melody() -> bool:
	var game := SpeculantGame.new()
	var started: Dictionary = JSON.parse_string(game.new_life(Saves.new(SAVES_ROOT).lobby()))
	if started.has("error"):
		return _fail("лобби не открылось: %s" % [started])
	var melody: Array = JSON.parse_string(game.view_json())["boot_melody"]
	if melody.is_empty() or melody[0]["hertz"] == null:
		return _fail("в снимке лобби нет мелодии заставки: %s" % [melody])
	var first: Dictionary = melody[0]
	var synth := SquareSynth.new(Speaker.MIX_RATE)
	synth.play(melody)
	var seconds := (float(first["milliseconds"]) - SquareSynth.GAP_MILLISECONDS) / SquareSynth.MILLISECONDS_IN_SECOND
	var measured := _rising_edges(synth.fill(roundi(seconds * Speaker.MIX_RATE))) / seconds
	if absf(measured - float(first["hertz"])) > float(first["hertz"]) * TOLERANCE:
		return _fail("первая нота мелодии %s Гц звучит на %.1f Гц" % [first["hertz"], measured])
	print("sound_smoke: мелодия заставки — %d тонов, первая нота %s Гц" % [melody.size(), first["hertz"]])
	return _write_wav(melody)


## Мелодия и бипы события, пейджера и отказа через паузы — 16 бит моно, как их выдаёт синтезатор.
func _write_wav(melody: Array) -> bool:
	var arguments := OS.get_cmdline_user_args()
	if arguments.size() <= OUTPUT_ARGUMENT:
		return _fail("ожидается путь WAV после --")
	var tones: Array = melody.duplicate()
	for cue_name: String in [Speaker.EVENT, Speaker.PAGER, Speaker.ERROR]:
		tones.append(LISTENING_PAUSE)
		tones.append_array(Speaker.CUES[cue_name])
	var milliseconds := 0.0
	for tone: Dictionary in tones:
		milliseconds += float(tone["milliseconds"])
	var synth := SquareSynth.new(Speaker.MIX_RATE)
	synth.play(tones)
	var frames := synth.fill(roundi(milliseconds * Speaker.MIX_RATE / SquareSynth.MILLISECONDS_IN_SECOND))
	var samples := PackedByteArray()
	samples.resize(frames.size() * 2)
	for index in frames.size():
		samples.encode_s16(index * 2, roundi(frames[index].x * SAMPLE_SCALE))
	var wav := AudioStreamWAV.new()
	wav.format = AudioStreamWAV.FORMAT_16_BITS
	wav.mix_rate = roundi(Speaker.MIX_RATE)
	wav.stereo = false
	wav.data = samples
	var saved := wav.save_to_wav(arguments[OUTPUT_ARGUMENT])
	if saved != OK:
		return _fail("WAV не записан в %s: %s" % [arguments[OUTPUT_ARGUMENT], error_string(saved)])
	return true


## Пищалка в дереве: после бипа генератор получает кадры, бип звучит, а выключенная — молчит.
func _check_speaker() -> void:
	_speaker = Speaker.new()
	root.add_child(_speaker)
	_speaker.cue(Speaker.EVENT)
	for _frame in FRAMES_TO_FILL:
		await process_frame
	if not _speaker.get_stream_playback() is AudioStreamGeneratorPlayback or not _speaker.playing:
		_fail("Speaker не запустил AudioStreamGenerator")
		return
	if not _speaker.is_sounding():
		_fail("бип события кончился, не прозвучав")
		return
	_speaker.set_enabled(false)
	_speaker.cue(Speaker.ERROR)
	if _speaker.is_sounding():
		_fail("выключенная пищалка играет бип")
		return
	print("sound_smoke: прямоугольная волна %s Гц, пауза — тишина, Speaker заполняет AudioStreamGenerator" % [HERTZ])
	_speaker.stop()
	for _frame in RELEASE_FRAMES:
		await process_frame
	_speaker.queue_free()
	await process_frame
	quit()


## Сколько раз волна перешла из минуса в плюс.
func _rising_edges(frames: PackedVector2Array) -> int:
	var edges := 0
	for index in range(1, frames.size()):
		if frames[index - 1].x < 0.0 and frames[index].x > 0.0:
			edges += 1
	return edges


func _fail(problem: String) -> bool:
	push_error("sound_smoke: " + problem)
	quit(1)
	return false
