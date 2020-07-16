#version 450

out vec3 tangent;
out vec2 encoded;
out vec2 uv_coords;
out vec3 v_position;
out vec3 base_normal;
out vec3 o_local_normal;

uniform int quads;
uniform int depth;
uniform float time;
uniform vec3 origin;
uniform float radius;
uniform mat4 transform;
uniform mat4 projection;
uniform vec2 chunk_coords;
uniform sampler2D heightmap;

const uvec2 quad[6] = {
  {1, 1}, {1, 0}, {0, 0},
  {0, 1}, {1, 1}, {0, 0},
};

vec3 unit_to_sphere(vec2 unit) {
  float side_length = 2.0 / pow(2, depth);
  return normalize(vec3((chunk_coords + unit) * side_length - 1.0, 1.0));
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
  vec3 result = (radius * local_normal - origin);

  gl_Position = projection * transform * vec4(result, 1.0);
  v_position = gl_Position.xyz / gl_Position.w;

  uv_coords = unit_coord;
  base_normal = mat3(transform) * local_normal;
  o_local_normal = local_normal;
}
