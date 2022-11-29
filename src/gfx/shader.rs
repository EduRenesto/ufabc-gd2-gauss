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
}
