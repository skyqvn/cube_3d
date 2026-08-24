use crate::texture::{ATLAS_COLS, ATLAS_ROWS};
use bytemuck::{Pod, Zeroable};

const CUBE_HALF: f32 = 0.5;
const VERTS_PER_FACE: u32 = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn create_cube_vertices() -> (Vec<Vertex>, Vec<u32>) {
    #[rustfmt::skip]
    let faces: [(usize, usize, [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]); 6] = [
        (0, 0, [ 0.0,  0.0,  CUBE_HALF], [ 1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-CUBE_HALF, -CUBE_HALF, CUBE_HALF], [ CUBE_HALF, -CUBE_HALF, CUBE_HALF], [ CUBE_HALF,  CUBE_HALF, CUBE_HALF], [-CUBE_HALF,  CUBE_HALF, CUBE_HALF]),
        (1, 0, [ 0.0,  0.0, -CUBE_HALF], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [ CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [-CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [-CUBE_HALF,  CUBE_HALF,-CUBE_HALF], [ CUBE_HALF,  CUBE_HALF,-CUBE_HALF]),
        (2, 0, [ CUBE_HALF, 0.0,  0.0], [ 0.0, 0.0,-1.0], [0.0, 1.0, 0.0], [ CUBE_HALF, -CUBE_HALF, CUBE_HALF], [ CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [ CUBE_HALF,  CUBE_HALF,-CUBE_HALF], [ CUBE_HALF,  CUBE_HALF, CUBE_HALF]),
        (0, 1, [-CUBE_HALF, 0.0,  0.0], [ 0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [-CUBE_HALF, -CUBE_HALF, CUBE_HALF], [-CUBE_HALF,  CUBE_HALF, CUBE_HALF], [-CUBE_HALF,  CUBE_HALF,-CUBE_HALF]),
        (1, 1, [ 0.0,  CUBE_HALF, 0.0], [ 1.0, 0.0, 0.0], [0.0, 0.0,-1.0], [-CUBE_HALF,  CUBE_HALF, CUBE_HALF], [ CUBE_HALF,  CUBE_HALF, CUBE_HALF], [ CUBE_HALF,  CUBE_HALF,-CUBE_HALF], [-CUBE_HALF,  CUBE_HALF,-CUBE_HALF]),
        (2, 1, [ 0.0, -CUBE_HALF, 0.0], [ 1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [ CUBE_HALF, -CUBE_HALF,-CUBE_HALF], [ CUBE_HALF, -CUBE_HALF, CUBE_HALF], [-CUBE_HALF, -CUBE_HALF, CUBE_HALF]),
    ];

    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (fi, (col, row, _center, _right, _up, p0, p1, p2, p3)) in faces.iter().enumerate() {
        let col = *col as f32;
        let row = *row as f32;
        let u0 = col / ATLAS_COLS as f32;
        let u1 = (col + 1.0) / ATLAS_COLS as f32;
        let v0 = row / ATLAS_ROWS as f32;
        let v1 = (row + 1.0) / ATLAS_ROWS as f32;

        let base = (fi * VERTS_PER_FACE as usize) as u32;
        vertices.push(Vertex {
            position: *p0,
            tex_coords: [u0, v1],
        });
        vertices.push(Vertex {
            position: *p1,
            tex_coords: [u1, v1],
        });
        vertices.push(Vertex {
            position: *p2,
            tex_coords: [u1, v0],
        });
        vertices.push(Vertex {
            position: *p3,
            tex_coords: [u0, v0],
        });

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    (vertices, indices)
}
