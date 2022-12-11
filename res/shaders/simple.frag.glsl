#version 450 core

in vec3 out_position;
in vec3 out_normal;

out vec4 frag_color;

void main() {
    frag_color = vec4(out_normal, 1.0);

    //vec3 u = abs(dFdxFine(out_normal));
    //vec3 v = abs(dFdyFine(out_normal));

    //float lu = 10000000 * length(u);
    //float lv = 10000000 * length(v);

    //frag_color = vec4(lu, lv, 1.0, 1.0);
}
