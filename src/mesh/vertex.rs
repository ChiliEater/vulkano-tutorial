use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex3d {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}