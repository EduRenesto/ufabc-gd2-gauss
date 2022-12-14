//! # Visualizador
//!
//! Este arquivo contém a integração entre a parte de computação
//! gráfica e a parte de cálculos.
//!
//! Não está inteiramente comentado porque não é o foco da disciplina.
//!
//! Mas, basicamente, esse arquivo contém a struct [`Viewer`], que configura
//! o estado da placa de vídeo para a renderização, carrega a malha
//! a partir do arquivo `.obj`, chama as funções de cálculo do módulo
//! [`crate::geom`], e faz a renderização.

use std::{f32::consts::PI, time::Duration};

use ultraviolet::Vec3;

use crate::gfx::{Shader, VertexBuffer};

/// A struct `Viewer` armazena o estado da aplicação.
pub struct Viewer<'a> {
    /// Contexto OpenGL
    gl: &'a glow::Context,

    /// O shader -- programa que é executado na placa de vídeo
    /// para determinar a renderização
    shader: Shader,

    /// Buffer de vértices, normais e curvaturas que é enviado
    /// para a placa de vídeo
    vao: VertexBuffer,

    /// Matriz que representa a transformação de espaço de objeto -> espaço de
    /// tela para a projeção
    cam_matrix: ultraviolet::Mat4,

    /// Matriz que representa a transformação ortogonal do modelo
    model_matrix: ultraviolet::Mat4,

    /// Ângulo de rotação do modelo
    rot: f32,
}

impl<'a> Viewer<'a> {
    /// Constrói um novo `Viewer`.
    pub fn new(gl: &'a glow::Context) -> Viewer {
        // Carrega o shader
        let shader = Shader::new(&gl,
            include_str!("../res/shaders/simple.frag.glsl"),
            include_str!("../res/shaders/simple.vert.glsl"),
        ).expect("failed to load shader");

        // Carrega o modelo.
        let mut load_opts = tobj::GPU_LOAD_OPTIONS;
        load_opts.single_index = false;

        let (models, _) = tobj::load_obj(
            "res/models/suzanne.obj",
            &load_opts,
        ).expect("failed to load model");

        // Constrói o buffer.
        // Aqui que chamamos as computações.
        let vao = {
            let model = &models[0];
            let mesh = &model.mesh;

            // Calcula as vizinhanças.
            let nbhds = crate::geom::compute_neighborhoods(mesh);
            // Calcula as normais médias.
            let raw_avg_normals = crate::geom::compute_avg_normals(mesh);
            // Calcula as bases dos planos tangentes.
            let tangent_basii = crate::geom::compute_tangent_basis(mesh, &nbhds, &raw_avg_normals);
            // Calcula as matrizes dos shape operators.
            let shape_ops = crate::geom::compute_shape_operator(mesh, &nbhds, &tangent_basii);
            // Calcula as curvaturas gaussianas e normais.
            let raw_curvatures = crate::geom::compute_curvatures(&shape_ops);

            let raw_positions = &mesh.positions;
            let raw_normals = &mesh.normals;

            // Prepara os vértices de um modo que a placa de vídeo espera.
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

            // Prepara as normais de um modo que a placa de vídeo espera.
            let _normals = mesh
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

            // Prepara as normais médias de um modo que a placa de vídeo espera.
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

            // Prepara as curvaturas gaussianas de um modo que a placa de vídeo espera.
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

            // Constrói o buffer e copia os dados para a placa de vídeo.
            VertexBuffer::from_mesh(gl, vertices, Some(avg_normals), Some(curvatures))
        };

        // Constrói a matriz da transformação da projeção.
        let cam_matrix = {
            let view = ultraviolet::Mat4::look_at(
                // Observador está no ponto (2, 2, 2) ...
                Vec3::new(2.0, 2.0, 2.0),
                // ... olhando para o ponto (0, 0, 0) ...
                Vec3::zero(),
                // ... e o topo de sua cabeça aponta para (0, 1, 0).
                Vec3::new(0.0, 1.0, 0.0),
            );

            let projection = ultraviolet::projection::perspective_gl(
                // Campo de visão de PI / 3...
                PI / 3.0,
                // ... com aspecto de 16/9 (widescreen) ...
                16.0/9.0,
                // ... e considerando tudo numa distância entre 1.0 ...
                1.0,
                // ... e 100.0.
                100.0,
            );

            projection * view
        };

        Viewer {
            gl,
            shader,
            vao,
            cam_matrix,
            model_matrix: ultraviolet::Mat4::identity(),
            rot: 0.0,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        let speed = 0.3; // 0.1 rad/s

        self.rot += speed * delta.as_secs_f32();

        self.model_matrix = ultraviolet::Mat4::from_euler_angles(
            0.0,
            self.rot,
            0.0,
        );
    }

    /// Renderiza a cena.
    pub fn render(&self) {
        self.shader.bind(self.gl);
        self.shader.uniform(self.gl, "_camera_mtx", &self.cam_matrix);
        self.shader.uniform(self.gl, "_model_mtx", &self.model_matrix);

        self.vao.draw(self.gl);
    }
}
