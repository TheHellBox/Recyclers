#version 450

out vec3 tangent;
out vec2 encoded;
out float height;
out vec2 uv_coords;
out vec3 v_position;
out vec3 base_normal;

uniform int quads;
uniform int depth;
uniform float time;
uniform vec3 origin;
uniform float radius;
uniform mat4 transform;
uniform mat4 projection;
uniform vec2 chunk_coords;

const uvec2 quad[6] = {
  {1, 1}, {1, 0}, {0, 0},
  {0, 1}, {1, 1}, {0, 0},
};

vec3 unit_to_sphere(vec2 unit) {
  float side_length = 2.0 / pow(2, depth);
  return normalize(vec3((chunk_coords + unit) * side_length - 1.0, 1.0));
}

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

float fbm(vec3 x) {
	float v = 0.0;
	float a = 0.5;
	vec3 shift = vec3(100);
	for (int i = 0; i < 12; ++i) {
		v += a * noise(x);
		x = x * 2.0 + shift;
		a *= 0.7;
	}
	return v;
}

void main() {
  int quad_id = gl_VertexID / 6;
  vec2 quad_coord = uvec2(quad_id % quads, quad_id / quads);
  vec2 position = quad[gl_VertexID % 6] + quad_coord;

  vec2 unit_coord = position / quads;
  float offset = 0.5 / (quads+1);
  float range = 1.0 - (2.0 * offset);

  vec2 h_coords = unit_coord * range + vec2(offset, offset);

  vec3 local_normal = unit_to_sphere(unit_coord);
  vec3 pos = (radius * local_normal - origin);
  float rb = 1.0 - fbm(local_normal * 1500 + vec3(time / 1000));
  height = rb;

  vec3 result = pos + rb * 20 * local_normal;

  gl_Position = projection * transform * vec4(result, 1.0);
  v_position = gl_Position.xyz / gl_Position.w;

  uv_coords = unit_coord;
  base_normal = mat3(transform) * local_normal;
}
