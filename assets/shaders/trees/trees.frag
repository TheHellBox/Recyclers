#version 450

in vec2 uv_coords;
in vec3 o_normal;

out vec4 color;

uniform sampler2D tex;
uniform sampler2D depth;
uniform mat4 transform;
uniform mat4 node_transform;

void main() {
  vec3 u_light = vec3(0.0, 1.0, 0.0);
  float brightness = dot(normalize(mat3(transform * node_transform) * o_normal), normalize(u_light));
  vec4 d = vec4(vec3(mix(0.1, 1.0, brightness)), 1.0);
  color = texture(tex, uv_coords) * d;
}
