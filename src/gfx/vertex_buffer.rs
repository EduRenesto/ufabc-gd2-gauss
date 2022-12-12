use core::slice;

use glow::{Context, HasContext};
use ultraviolet::{ Vec3, Vec2 };

pub struct VertexBuffer {
    vao: glow::VertexArray,
    n_vertices: usize,
}

impl VertexBuffer {
    pub fn from_mesh(
        gl: &Context,
        vertices: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
        gaussian_curvatures: Option<Vec<f32>>,
    ) -> VertexBuffer {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vec3>());

                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                gl.memory_barrier(glow::ALL_BARRIER_BITS);
            }

            if let Some(normals) = normals {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(normals.as_ptr() as *const u8, normals.len() * std::mem::size_of::<Vec3>());

                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(1);
                gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);

                gl.memory_barrier(glow::ALL_BARRIER_BITS);
            };

            if let Some(gaussian_curvatures) = gaussian_curvatures {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(gaussian_curvatures.as_ptr() as *const u8, gaussian_curvatures.len() * std::mem::size_of::<f32>());

                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(2);
                gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, 0, 0);

                gl.memory_barrier(glow::ALL_BARRIER_BITS);
            };

            VertexBuffer { vao, n_vertices: vertices.len() }
        }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, self.n_vertices as i32);
        }
    }
}
