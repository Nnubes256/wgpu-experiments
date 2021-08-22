#version 450

layout(location=0) in vec2 vec_pos;

layout(location=0) out vec4 f_color;

void main() {
    f_color = vec4(abs(vec_pos), 0.1, 1.0);
}