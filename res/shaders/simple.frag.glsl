#version 450 core

in vec3 out_position;
in vec3 out_normal;
in float out_curvature;

out vec4 frag_color;

void main() {
    const vec3 lo = vec3(0.0, 0.0, 1.0);
    const vec3 hi = vec3(1.0, 0.0, 0.0);

    vec3 wtf_color = mix(lo, hi, out_curvature);

    //frag_color = vec4(out_normal, 1.0);
    frag_color = vec4(wtf_color, 1.0);

    //vec3 u = abs(dFdxFine(out_normal));
    //vec3 v = abs(dFdyFine(out_normal));

    //float lu = 10000000 * length(u);
    //float lv = 10000000 * length(v);

    //frag_color = vec4(lu, lv, 1.0, 1.0);
}
