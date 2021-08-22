#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(location=5) in vec4 i_model_matrix_0;
layout(location=6) in vec4 i_model_matrix_1;
layout(location=7) in vec4 i_model_matrix_2;
layout(location=8) in vec4 i_model_matrix_3;

layout(set=1, binding=0) uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_tex_coords = a_tex_coords;
    mat4 i_model_matrix = mat4(
        i_model_matrix_0,
        i_model_matrix_1,
        i_model_matrix_2,
        i_model_matrix_3
    );
    gl_Position = u_view_proj * i_model_matrix * vec4(a_position, 1.0);
}
