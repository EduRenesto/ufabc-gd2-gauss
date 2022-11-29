#version 430 core

/*
 * This is where all of the lighting calculations happen.
 * The previous render passes only populated the GBuffer.
 * In this shader, we sample the textures of the GBuffer and
 * finally do all the light calculations.
 */

in vec2 out_tex_coord;

/*
 * These are the textures from the GBuffer.
 */
uniform sampler2D _positions_texture;
uniform sampler2D _normals_texture;
uniform sampler2D _diffuse_texture;

uniform vec3 _camera_pos;

/*
 * A cool thing about deferred shading is that it allows for
 * multiple lights easily.
 *
 * Here, we define an arbitrary array of lights. We need to separately
 * declare the number of lights and the array because all array sizes in
 * GLSL must be static. Here, 16 is a hardcoded value that I thought was
 * enough, but this can easily be adjusted if needed.
 */
uniform int _total_lights;
uniform vec3 _light_positions[16];

out vec4 frag_color;

/*
 * Calculates the Phong shading model for a given fragment.
 */
vec3 phong(
  vec3 position,
  vec3 normal,
  vec3 light_position,
  vec3 view_position,
  float shininess,
  vec3 diffuse_color
) {
  vec3 ld = normalize(light_position - position);

  float lambertian = clamp(dot(normal, ld), 0.0, 1.0);
  vec3 diffuse_term = lambertian * diffuse_color;

  float specular = 0.0;

  if (lambertian > 0.0) {
    vec3 refl = reflect(-ld, normal);
    vec3 view = normalize(light_position - _camera_pos);
    float angle = max(dot(refl, view), 0.0);
    specular = pow(angle, shininess);
  }

  float d = length(light_position - position);
  float attenuation = 1.0 / (1.0 + 0.0005 * d + 0.0005 * d * d);

  return attenuation * (diffuse_term + vec3(specular));
}

void main() {
  /*
   * Here, we sample each of the GBuffer textures, and store the corresponding
   * data in a separate variable, to avoid resampling every time.
   */
  vec3 diffuse_color = texture(_diffuse_texture, out_tex_coord).xyz;
  float shininess = texture(_diffuse_texture, out_tex_coord).w;
  vec3 normal = texture(_normals_texture, out_tex_coord).xyz;
  vec3 position = texture(_positions_texture, out_tex_coord).xyz;

  /*
   * We run this loop through all the lights passed to the shader,
   * accumulating the output in the `throughput` variable.
   *
   * Easy as 1,2,3!
   */
  vec3 throughput;
  for (int i = 0; i < _total_lights; i++) {
    throughput += phong(
      position,
      normal,
      _light_positions[i],
      _camera_pos,
      shininess,
      diffuse_color
    );
  }

  /*
   * Finally, just write the final calculations to the fragment color.
   *
   * Look how we only did O(m * n) shading calculations, where `m` is
   * the number of fragments in the fullscreen quad and `n` is the number
   * of lights.
   *
   * Compare this to if we had to run the Phong model on every single fragment
   * that's been rendered.
   *
   * Since our scene is relatively simple, maybe the payoff isn't that high.
   * But, on scenes with a lot of meshes and different light sources, deferred
   * rendering is the way to go.
   */
  frag_color = vec4(throughput, 1.0);
}
