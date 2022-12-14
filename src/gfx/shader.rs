//! # Shader
//!
//! Um shader é um programa que é executado na GPU, e determina
//! as posições que a GPU deve desenhar, e como pintar cada pixel.
//!
//! Esse módulo não contém nada de especial, e só serve como utilidade
//! para remover o boilerplate do código principal do projeto.

use glow::{Context, HasContext};

pub struct Shader {
    program: glow::Program,
}

impl Shader {
    pub fn new(
        gl: &Context,
        frag_src: &str,
        vert_src: &str,
    ) -> Option<Self> {
        unsafe {
            let program = gl.create_program().unwrap();

            let frag = {
                let frag = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
                gl.shader_source(frag, frag_src);

                gl.compile_shader(frag);

                if !gl.get_shader_compile_status(frag) {
                    let msg = gl.get_shader_info_log(frag);

                    println!("Failed to compile fragment shader!");
                    print!("{}", msg);

                    None
                } else {
                    Some(frag)
                }
            }?;

            let vert = {
                let vert = gl.create_shader(glow::VERTEX_SHADER).unwrap();
                gl.shader_source(vert, vert_src);

                gl.compile_shader(vert);

                if !gl.get_shader_compile_status(vert) {
                    let msg = gl.get_shader_info_log(vert);

                    println!("Failed to compile vert shader!");
                    print!("{}", msg);

                    None
                } else {
                    Some(vert)
                }
            }?;

            gl.attach_shader(program, frag);
            gl.attach_shader(program, vert);

            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let msg = gl.get_program_info_log(program);

                println!("Failed to link shader program!");
                print!("{}", msg);

                None
            } else {
                Some(Shader {
                    program,
                })
            }
        }
    }

    pub fn bind(&self, gl: &Context) {
        unsafe { gl.use_program(Some(self.program)) };
    }

    pub fn uniform<T: Uniform>(&self, gl: &Context, name: &str, val: &T) {
        unsafe {
            let loc = gl
                .get_uniform_location(self.program, name)
                .expect(&format!("Uniform {} not found", name));

            val.bind(gl, &loc);
        }
    }
}

pub trait Uniform {
    unsafe fn bind(&self, gl: &glow::Context, loc: &glow::UniformLocation);
}

impl Uniform for ultraviolet::Mat4 {
    unsafe fn bind(&self, gl: &glow::Context, loc: &glow::UniformLocation) {
        gl.uniform_matrix_4_f32_slice(Some(loc), false, self.as_slice());
    }
}
