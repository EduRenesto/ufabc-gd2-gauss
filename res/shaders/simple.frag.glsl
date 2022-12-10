#version 430 core

in vec3 out_position;
in vec3 out_normal;

out vec4 frag_color;

void main() {
    frag_color = vec4(out_normal, 1.0);
}
