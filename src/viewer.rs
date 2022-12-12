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

        let mut load_opts = tobj::GPU_LOAD_OPTIONS;
        load_opts.single_index = false;

        let (models, _) = tobj::load_obj(
            "res/models/torus.obj",
            &load_opts,
        ).expect("failed to load model");

        let vao = {
            let model = &models[0];
            let mesh = &model.mesh;

            let nbhds = crate::geom::compute_neighborhoods(mesh);
            let raw_avg_normals = crate::geom::compute_avg_normals(mesh);
            let tangent_basii = crate::geom::compute_tangent_basis(mesh, &nbhds, &raw_avg_normals);
            let shape_ops = crate::geom::compute_shape_operator(mesh, &nbhds, &raw_avg_normals, &tangent_basii);
            let raw_curvatures = crate::geom::compute_curvatures(&shape_ops);

            let raw_positions = &mesh.positions;
            let raw_normals = &mesh.normals;

            let vertices = mesh
                .indices
                .chunks_exact(3)
                .map(|idxs| {
                    let v1 = Vec3::new(
                        raw_positions[3 * idxs[0] as usize + 0],
                        raw_positions[3 * idxs[0] as usize + 1],
                        raw_positions[3 * idxs[0] as usize + 2],
                    );
                    let v2 = Vec3::new(
                        raw_positions[3 * idxs[1] as usize + 0],
                        raw_positions[3 * idxs[1] as usize + 1],
                        raw_positions[3 * idxs[1] as usize + 2],
                    );
                    let v3 = Vec3::new(
                        raw_positions[3 * idxs[2] as usize + 0],
                        raw_positions[3 * idxs[2] as usize + 1],
                        raw_positions[3 * idxs[2] as usize + 2],
                    );

                    [v1, v2, v3]
                })
                .flatten()
                .collect::<Vec<_>>();

            let normals = mesh
                .normal_indices
                .chunks_exact(3)
                .map(|idxs| {
                    let v1 = Vec3::new(
                        raw_normals[3 * idxs[0] as usize + 0],
                        raw_normals[3 * idxs[0] as usize + 1],
                        raw_normals[3 * idxs[0] as usize + 2],
                    );
                    let v2 = Vec3::new(
                        raw_normals[3 * idxs[1] as usize + 0],
                        raw_normals[3 * idxs[1] as usize + 1],
                        raw_normals[3 * idxs[1] as usize + 2],
                    );
                    let v3 = Vec3::new(
                        raw_normals[3 * idxs[2] as usize + 0],
                        raw_normals[3 * idxs[2] as usize + 1],
                        raw_normals[3 * idxs[2] as usize + 2],
                    );

                    [v1, v2, v3]
                })
                .flatten()
                .collect::<Vec<_>>();

            let avg_normals = mesh
                .indices
                .chunks_exact(3)
                .map(|idxs| {
                    let n1 = raw_avg_normals[idxs[0] as usize];
                    let n2 = raw_avg_normals[idxs[1] as usize];
                    let n3 = raw_avg_normals[idxs[2] as usize];

                    [n1, n2, n3]
                })
                .flatten()
                .collect::<Vec<_>>();

            let curvatures = mesh
                .indices
                .chunks_exact(3)
                .map(|idxs| {
                    let k1 = raw_curvatures[idxs[0] as usize].0;
                    let k2 = raw_curvatures[idxs[1] as usize].0;
                    let k3 = raw_curvatures[idxs[2] as usize].0;

                    [k1, k2, k3]
                })
                .flatten()
                .collect::<Vec<_>>();

            println!("raw_curvatures.len = {}, raw_vertices.len = {}", raw_curvatures.len(), raw_positions.len());

            //VertexBuffer::from_mesh(gl, vertices, Some(normals), None)
            VertexBuffer::from_mesh(gl, vertices, Some(avg_normals), Some(curvatures))
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
