use core::slice;

use glow::{Context, HasContext};
use ultraviolet::{ Vec3, Vec2 };

pub struct VertexBuffer {
    vao: glow::VertexArray,
    ibo: glow::Buffer,
    n_vertices: usize,
}

impl VertexBuffer {
    pub fn from_mesh(
        gl: &Context,
        indices: Vec<u16>,
        vertices: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
        tex_coords: Option<Vec<Vec2>>,
    ) -> VertexBuffer {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let ibo = gl.create_buffer().unwrap();
            {
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
                let data = slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * std::mem::size_of::<u16>());

                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, glow::STATIC_DRAW);
            }

            {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::VERTEX_ARRAY, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vec3>());

                gl.buffer_data_u8_slice(glow::VERTEX_ARRAY, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                // HACK forgive me, father, for I have sinned.
                std::mem::forget(vbo);
            }

            if let Some(normals) = normals {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::VERTEX_ARRAY, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(normals.as_ptr() as *const u8, normals.len() * std::mem::size_of::<Vec3>());

                gl.buffer_data_u8_slice(glow::VERTEX_ARRAY, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(1);
                gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);

                std::mem::forget(vbo);
            };

            if let Some(tex_coords) = tex_coords {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::VERTEX_ARRAY, Some(vbo));

                // HACK
                let data  = slice::from_raw_parts(tex_coords.as_ptr() as *const u8, tex_coords.len() * std::mem::size_of::<Vec3>());

                gl.buffer_data_u8_slice(glow::VERTEX_ARRAY, data, glow::STATIC_DRAW);

                gl.enable_vertex_attrib_array(2);
                gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 0, 0);

                std::mem::forget(vbo);
            };

            VertexBuffer { ibo, vao, n_vertices: indices.len() }
        }
    }

    pub fn draw(&self, gl: &Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));

            gl.draw_elements(glow::TRIANGLES, self.n_vertices as i32, glow::UNSIGNED_SHORT, 0);
            //gl.draw_arrays(glow::TRIANGLES, 0, self.n_vertices as i32);
        }
    }
}
