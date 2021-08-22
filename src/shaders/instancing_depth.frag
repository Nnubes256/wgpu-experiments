#version 450

layout(location = 0) in vec2 v_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform samplerShadow s_diffuse;

void main() {
    float near = 0.1;
    float far = 100.0;

    // This returns a number between 0 and 1
    float depth = texture(sampler2DShadow(t_diffuse, s_diffuse), vec3(v_tex_coords, 1));

    // This normalizes it to the [-1, 1] scale
    depth = 2.0 * depth - 1.0;

    // Gradient definition
    float color = (2.0 * near) / (far + near - depth * (far - near));

    f_color = vec4(vec3(depth), 1.0);
}
