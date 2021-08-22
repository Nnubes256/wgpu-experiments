#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;

layout(location = 0) out vec2 vec_pos;

void main() {
    vec_pos = vec2(a_position.x, a_position.y);
    gl_Position = vec4(a_position, 1.0);
}
