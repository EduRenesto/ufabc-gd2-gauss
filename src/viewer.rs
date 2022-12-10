use std::{io::BufReader, fs::File, f32::consts::PI};

use ultraviolet::Vec3;

use crate::gfx::{Shader, VertexBuffer};

pub struct Viewer<'a> {
    gl: &'a glow::Context,

    shader: Shader,
    vao: VertexBuffer,

    cam_matrix: ultraviolet::Mat4,
}

impl<'a> Viewer<'a> {
    pub fn new(gl: &'a glow::Context) -> Viewer {
        let shader = Shader::new(&gl,
            include_str!("../res/shaders/simple.frag.glsl"),
            include_str!("../res/shaders/simple.vert.glsl"),
        ).expect("failed to load shader");

        let file = BufReader::new(File::open("res/models/torus.obj").unwrap());
        let model: obj::Obj = obj::load_obj(file).expect("failed to load model");

        let vao = {
            let indices = model.indices;

            let vertices = model.vertices.iter().map(|vtx| {
                let [x,y,z] = vtx.position;

                ultraviolet::Vec3::new(x,y,z)
            }).collect::<Vec<_>>();

            let normals = model.vertices.iter().map(|vtx| {
                let [x,y,z] = vtx.normal;

                ultraviolet::Vec3::new(x,y,z)
            }).collect::<Vec<_>>();

            VertexBuffer::from_mesh(gl, indices, vertices, Some(normals), None)
        };

        let cam_matrix = {
            let view = ultraviolet::Mat4::look_at(Vec3::new(2.0, 2.0, 2.0), Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
            let projection = ultraviolet::projection::perspective_gl(PI / 3.0, 16.0/9.0, 1.0, 100.0);

            projection * view
        };

        Viewer {
            gl,
            shader,
            vao,
            cam_matrix,
        }
    }

    pub fn render(&self) {
        self.shader.bind(self.gl);
        self.shader.uniform(self.gl, "_camera_mtx", &self.cam_matrix);

        self.vao.draw(self.gl);
    }
}
