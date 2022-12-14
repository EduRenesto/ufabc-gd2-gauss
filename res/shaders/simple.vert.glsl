#version 410 core

/*
 * This shader is only used to render the fullscren quad itself.
 * It doesn't do any shading whatsoever.
 */

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in float in_curvature;

out vec3 out_position;
out vec3 out_normal;
out float out_curvature;

uniform mat4 _camera_mtx;
uniform mat4 _model_mtx;

void main() {
    mat4 mvp = _camera_mtx * _model_mtx;
    gl_Position = mvp * vec4(in_position, 1.0);

    out_position = gl_Position.xyz;
    out_normal = in_normal;

    out_curvature = (clamp(in_curvature, -5, 5) / 5 + 1)/2;
}
