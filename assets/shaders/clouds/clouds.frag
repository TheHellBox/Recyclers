#version 450

in vec3 tangent;
in vec2 uv_coords;
in vec3 v_position;
in vec3 base_normal;
in vec3 o_local_normal;

out vec4 color;

uniform mat4 view;
uniform int depth;
uniform float time;
uniform vec3 origin;
uniform bool inverse;
uniform vec4 tex_color;
uniform vec2 chunk_coords;
uniform sampler2D heightmap;

float mod289(float x){return x - floor(x * (1.0 / 289.0)) * 289.0;}
vec4 mod289(vec4 x){return x - floor(x * (1.0 / 289.0)) * 289.0;}
vec4 perm(vec4 x){return mod289(((x * 34.0) + 1.0) * x);}

float noise(vec3 p){
    vec3 a = floor(p);
    vec3 d = p - a;
    d = d * d * (3.0 - 2.0 * d);

    vec4 b = a.xxyy + vec4(0.0, 1.0, 0.0, 1.0);
    vec4 k1 = perm(b.xyxy);
    vec4 k2 = perm(k1.xyxy + b.zzww);

    vec4 c = k2 + a.zzzz;
    vec4 k3 = perm(c);
    vec4 k4 = perm(c + 1.0);

    vec4 o1 = fract(k3 * (1.0 / 41.0));
    vec4 o2 = fract(k4 * (1.0 / 41.0));

    vec4 o3 = o2 * d.z + o1 * (1.0 - d.z);
    vec2 o4 = o3.yw * d.x + o3.xz * (1.0 - d.x);

    return o4.y * d.y + o4.x * (1.0 - d.y);
}

float fbm(vec3 x, int octaves) {
	float v = 0.0;
	float a = 0.5;
	for (int i = 0; i < octaves; ++i) {
		v += a * noise(x);
		x = x * 2.0;
		a *= 0.7;
	}
	return v;
}

void main() {
  vec3 base_normal_ = normalize(base_normal);
  vec3 camera_dir = normalize(-v_position);
  vec3 sun = mat3(view) * vec3(0, -1, 0);
  float b = clamp(dot(base_normal_, sun), 0, 1);
  vec4 h = mix(vec4(0.001, 0.001, 0.001, 1), vec4(1, 1, 1, 1), b);

  vec3 normal = base_normal_ * mat3(view);
  vec3 r = vec3(0.0);
  r.x = fbm(normal + vec3(time) / 1000, 4);
  r.y = fbm(normal + vec3(time) / 1500, 4);
  r.z = fbm(normal + vec3(time) / 1200, 4);

  float shade = fbm(r + normal + vec3(time) / 2000, 12) * 0.9;
  float alpha = mix(shade, 0.0, 1.3 - shade);
  if (inverse) {
    shade = 1.0 - shade;
  }
  
  vec4 c = vec4(shade);
  c.a = alpha;
  color = c * h;
}
