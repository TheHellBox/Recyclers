#version 450

in vec3 tangent;
in float height;
in vec2 uv_coords;
in vec3 v_position;
in vec3 base_normal;

out vec4 color;

uniform mat4 view;
uniform int depth;
uniform float time;
uniform vec4 tex_color;
uniform vec2 chunk_coords;
uniform sampler2D heightmap;

void main() {
  vec3 base_normal_ = normalize(base_normal);
  vec3 camera_dir = normalize(-v_position);

  vec3 sun = vec3(0, -1, 0) * mat3(view);
  float b = clamp(dot(base_normal_, sun), 0, 1);

  vec3 half_direction = normalize(sun + camera_dir);
  float specular = pow(max(dot(half_direction, base_normal_), 0.0), 16.0);

  vec4 h = mix(vec4(0.001, 0.001, 0.001, 1), vec4(1, 1, 1, 1), (b + specular));
  vec4 r = vec4(0.04, 0.15, 0.29, 0.95);
  color = r * h;
}
