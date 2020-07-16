#version 450

in vec2 uv;
in vec3 position;
in vec3 normal;

uniform mat4 view;
uniform mat4 transform;
uniform mat4 node_transform;
uniform mat4 projection;

out vec2 uv_coords;
out vec3 o_normal;

void main() {
  mat4 mat = projection * (transform * node_transform);
  uv_coords = uv;
  o_normal = normal;
  gl_Position = mat * vec4(position, 1.0);
}
