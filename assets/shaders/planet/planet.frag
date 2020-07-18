#version 450

in vec3 tangent;
in vec2 encoded;
in float height;
in vec2 uv_coords;
in vec3 base_normal;

out vec4 color;

uniform mat4 view;
uniform int depth;
uniform vec4 tex_color;
uniform sampler2DArray tex;

void main() {
  vec3 base_normal_ = normalize(base_normal);
  vec3 tangent_ = normalize(tangent);
  vec3 bitangent = cross(base_normal_, tangent_);
  mat3 tangent_basis = mat3(tangent_, bitangent, base_normal_);

  vec3 decoded = vec3(encoded, sqrt(1-dot(encoded.xy, encoded.xy)));
  vec3 normal = tangent_basis * decoded;

  vec3 sun = mat3(view) * vec3(0, -1, 0);
  float brightness= clamp(dot(normal, sun), 0.0, 1.0);
  float angle = dot(base_normal_, normal);

  vec4 h = mix(vec4(0.005, 0.005, 0.005, 1), vec4(1, 1, 1, 1), brightness);

  vec4 color1a = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 2000, 0) ));
  vec4 color1b = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 300, 0) ));
  vec4 color1 = mix(color1a, color1b, 0.2);

  vec4 color2a = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 2000, 1) ));
  vec4 color2b = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 300, 1) ));
  vec4 color2 = mix(color2a, color2b, 0.2);

  vec4 color3a = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 2000, 2) ));
  vec4 color3b = vec4(texture(tex, vec3(uv_coords / (depth * depth) * 300, 2) ));
  vec4 color3 = mix(color3a, color3b, 0.2);

  color = mix(color1, color2, clamp((0.01 - height) * 100, 0, 1)) * h;
  color = mix(color3, color, angle);
}
