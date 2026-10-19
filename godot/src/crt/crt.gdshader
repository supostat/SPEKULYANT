// CRT-эффект (docs/SPEKULYANT.md §13.1 п. 6) поверх готового кадра 80×25: каждая вторая строка
// пикселей темнее — развёртка, светлые пиксели чуть растекаются по горизонтали — люминофор, к углам
// кадр темнеет — виньетка. Изгиба нет: текст должен читаться так же, как без эффекта.
shader_type canvas_item;

uniform sampler2D screen_texture : hint_screen_texture, filter_nearest;
uniform float scanline_darkness : hint_range(0.0, 1.0) = 0.25;
uniform float glow : hint_range(0.0, 1.0) = 0.3;
uniform float vignette : hint_range(0.0, 1.0) = 0.35;
uniform float brightness : hint_range(1.0, 2.0) = 1.12;

void fragment() {
	vec2 step_x = vec2(SCREEN_PIXEL_SIZE.x, 0.0);
	vec3 color = texture(screen_texture, SCREEN_UV).rgb;
	vec3 neighbors = 0.5 * (texture(screen_texture, SCREEN_UV - step_x).rgb + texture(screen_texture, SCREEN_UV + step_x).rgb);
	color = mix(color, max(color, neighbors), glow);
	float odd_line = mod(floor(FRAGCOORD.y), 2.0);
	color *= 1.0 - scanline_darkness * odd_line;
	vec2 centered = SCREEN_UV - vec2(0.5);
	color *= 1.0 - vignette * dot(centered, centered);
	COLOR = vec4(color * brightness, 1.0);
}
