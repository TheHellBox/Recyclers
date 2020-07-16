#version 450

in vec2 uv;
in vec3 position;
in vec3 normal;
in vec2 tree_position;
in uint tree_type;

out vec2 uv_coords;
out vec3 o_normal;

uniform int quads;
uniform int depth;
uniform float time;
uniform vec3 origin;
uniform float radius;
uniform mat4 transform;
uniform mat4 node_transform;
uniform mat4 projection;
uniform vec2 chunk_coords;
uniform sampler2D heightmap;
uniform float max_height;

const uvec2 quad[6] = {
  {1, 1}, {1, 0}, {0, 0},
  {0, 1}, {1, 1}, {0, 0},
};

vec3 unit_to_sphere(vec2 unit) {
  float side_length = 2.0 / pow(2, depth);
  return normalize(vec3((chunk_coords + unit) * side_length - 1.0, 1.0));
}

void main() {
  vec3 local_normal = unit_to_sphere(tree_position);
  float h = texture(heightmap, tree_position).x;
  
  vec3 result = (radius * local_normal - origin) + h * max_height * local_normal;
  gl_Position = projection * transform * (vec4(result, 0.0) + node_transform * vec4(position, 1.0));

  uv_coords = uv;
  o_normal = normal;
}
