use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowBuilder},
};

const MOUSE_SENSITIVITY: f64 = 0.006;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model_view_proj: [[f32; 4]; 4],
}

fn create_cube_vertices() -> (Vec<Vertex>, Vec<u32>) {
    let half = 0.5f32;

    #[rustfmt::skip]
    let faces: [(usize, usize, [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]); 6] = [
        // (atlas_col, atlas_row, center,   right,     up,         p0,        p1,        p2,        p3)
        (0, 0, [ 0.0,  0.0,  half], [ 1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-half, -half, half], [ half, -half, half], [ half,  half, half], [-half,  half, half]), // front
        (1, 0, [ 0.0,  0.0, -half], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [ half, -half,-half], [-half, -half,-half], [-half,  half,-half], [ half,  half,-half]), // back
        (2, 0, [ half, 0.0,  0.0], [ 0.0, 0.0,-1.0], [0.0, 1.0, 0.0], [ half, -half, half], [ half, -half,-half], [ half,  half,-half], [ half,  half, half]), // right
        (0, 1, [-half, 0.0,  0.0], [ 0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-half, -half,-half], [-half, -half, half], [-half,  half, half], [-half,  half,-half]), // left
        (1, 1, [ 0.0,  half, 0.0], [ 1.0, 0.0, 0.0], [0.0, 0.0,-1.0], [-half,  half, half], [ half,  half, half], [ half,  half,-half], [-half,  half,-half]), // top
        (2, 1, [ 0.0, -half, 0.0], [ 1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-half, -half,-half], [ half, -half,-half], [ half, -half, half], [-half, -half, half]), // bottom
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (fi, (col, row, _center, _right, _up, p0, p1, p2, p3)) in faces.iter().enumerate() {
        let col = *col as f32;
        let row = *row as f32;
        let u0 = col / 3.0;
        let u1 = (col + 1.0) / 3.0;
        let v0 = row / 2.0;
        let v1 = (row + 1.0) / 2.0;

        let base = (fi * 4) as u32;
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

fn create_face_texture(color: [u8; 4], label: &str) -> Vec<u8> {
    let size: u32 = 256;
    let mut pixels = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let border = 4;
            if x < border || x >= size - border || y < border || y >= size - border {
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            } else {
                pixels[idx] = color[0];
                pixels[idx + 1] = color[1];
                pixels[idx + 2] = color[2];
                pixels[idx + 3] = color[3];
            }
        }
    }

    let font_8x8: [[u8; 8]; 96] = [
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00],
        [0x6C, 0x6C, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x6C, 0x6C, 0xFE, 0x6C, 0xFE, 0x6C, 0x6C, 0x00],
        [0x18, 0x7E, 0xC0, 0x7C, 0x06, 0xFC, 0x18, 0x00],
        [0x00, 0xC6, 0xCC, 0x18, 0x30, 0x66, 0xC6, 0x00],
        [0x38, 0x6C, 0x38, 0x76, 0xDC, 0xCC, 0x76, 0x00],
        [0x30, 0x30, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00],
        [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30],
        [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        [0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x80, 0x00],
        [0x7C, 0xCE, 0xDE, 0xF6, 0xE6, 0xE6, 0x7C, 0x00],
        [0x18, 0x38, 0x78, 0x18, 0x18, 0x18, 0x7E, 0x00],
        [0x7C, 0xC6, 0x06, 0x1C, 0x70, 0xC0, 0xFE, 0x00],
        [0x7C, 0xC6, 0x06, 0x3C, 0x06, 0xC6, 0x7C, 0x00],
        [0x1C, 0x3C, 0x6C, 0xCC, 0xFE, 0x0C, 0x0C, 0x00],
        [0xFE, 0xC0, 0xFC, 0x06, 0x06, 0xC6, 0x7C, 0x00],
        [0x3C, 0x60, 0xC0, 0xFC, 0xC6, 0xC6, 0x7C, 0x00],
        [0xFE, 0xC6, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00],
        [0x7C, 0xC6, 0xC6, 0x7C, 0xC6, 0xC6, 0x7C, 0x00],
        [0x7C, 0xC6, 0xC6, 0x7E, 0x06, 0x0C, 0x78, 0x00],
        [0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00],
        [0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x30],
        [0x0C, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0C, 0x00],
        [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00],
        [0x30, 0x18, 0x0C, 0x06, 0x0C, 0x18, 0x30, 0x00],
        [0x7C, 0xC6, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00],
        [0x7C, 0xC6, 0xDE, 0xDE, 0xDE, 0xC0, 0x7C, 0x00],
        [0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0x00],
        [0xFC, 0xC6, 0xC6, 0xFC, 0xC6, 0xC6, 0xFC, 0x00],
        [0x3C, 0x66, 0xC0, 0xC0, 0xC0, 0x66, 0x3C, 0x00],
        [0xF8, 0xCC, 0xC6, 0xC6, 0xC6, 0xCC, 0xF8, 0x00],
        [0xFE, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xFE, 0x00],
        [0xFE, 0xC0, 0xC0, 0xF8, 0xC0, 0xC0, 0xC0, 0x00],
        [0x3E, 0x60, 0xC0, 0xDE, 0xC6, 0x66, 0x3E, 0x00],
        [0xC6, 0xC6, 0xC6, 0xFE, 0xC6, 0xC6, 0xC6, 0x00],
        [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        [0x06, 0x06, 0x06, 0x06, 0x06, 0xC6, 0x7C, 0x00],
        [0xC6, 0xCC, 0xD8, 0xF0, 0xD8, 0xCC, 0xC6, 0x00],
        [0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xFE, 0x00],
        [0xC6, 0xEE, 0xFE, 0xD6, 0xC6, 0xC6, 0xC6, 0x00],
        [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0x00],
        [0x38, 0x6C, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x00],
        [0xFC, 0xC6, 0xC6, 0xFC, 0xC0, 0xC0, 0xC0, 0x00],
        [0x38, 0x6C, 0xC6, 0xC6, 0xD6, 0x6C, 0x3E, 0x00],
        [0xFC, 0xC6, 0xC6, 0xFC, 0xD8, 0xCC, 0xC6, 0x00],
        [0x7C, 0xC6, 0xC0, 0x7C, 0x06, 0xC6, 0x7C, 0x00],
        [0xFF, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        [0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0x7C, 0x00],
        [0xC6, 0xC6, 0xC6, 0xC6, 0x6C, 0x38, 0x10, 0x00],
        [0xC6, 0xC6, 0xC6, 0xD6, 0xFE, 0xEE, 0xC6, 0x00],
        [0xC6, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0xC6, 0x00],
        [0xC3, 0xC3, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
        [0xFE, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0xFE, 0x00],
        [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00],
        [0xC0, 0x60, 0x30, 0x18, 0x0C, 0x06, 0x02, 0x00],
        [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00],
        [0x10, 0x38, 0x6C, 0xC6, 0x00, 0x00, 0x00, 0x00],
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
        [0x30, 0x30, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x00, 0x00, 0x78, 0x0C, 0x7C, 0xCC, 0x76, 0x00],
        [0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xFC, 0x00],
        [0x00, 0x00, 0x7C, 0xC6, 0xC0, 0xC6, 0x7C, 0x00],
        [0x06, 0x06, 0x7E, 0xC6, 0xC6, 0xC6, 0x7E, 0x00],
        [0x00, 0x00, 0x7C, 0xC6, 0xFE, 0xC0, 0x7C, 0x00],
        [0x1C, 0x36, 0x30, 0xFC, 0x30, 0x30, 0x30, 0x00],
        [0x00, 0x00, 0x76, 0xCE, 0xC6, 0x7E, 0x06, 0x7C],
        [0xC0, 0xC0, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0x00],
        [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00],
        [0x0C, 0x00, 0x1C, 0x0C, 0x0C, 0xCC, 0xCC, 0x78],
        [0xC0, 0xC0, 0xCC, 0xD8, 0xF0, 0xD8, 0xCC, 0x00],
        [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        [0x00, 0x00, 0xEC, 0xFE, 0xD6, 0xD6, 0xD6, 0x00],
        [0x00, 0x00, 0xDC, 0xE6, 0xC6, 0xC6, 0xC6, 0x00],
        [0x00, 0x00, 0x7C, 0xC6, 0xC6, 0xC6, 0x7C, 0x00],
        [0x00, 0x00, 0xFC, 0xC6, 0xC6, 0xFC, 0xC0, 0xC0],
        [0x00, 0x00, 0x7E, 0xC6, 0xC6, 0x7E, 0x06, 0x06],
        [0x00, 0x00, 0xDC, 0xE6, 0xC0, 0xC0, 0xC0, 0x00],
        [0x00, 0x00, 0x7C, 0xC0, 0x7C, 0x06, 0x7C, 0x00],
        [0x30, 0x30, 0xFC, 0x30, 0x30, 0x36, 0x1C, 0x00],
        [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0xC6, 0x7E, 0x00],
        [0x00, 0x00, 0xC6, 0xC6, 0x6C, 0x38, 0x10, 0x00],
        [0x00, 0x00, 0xC6, 0xD6, 0xD6, 0xFE, 0x6C, 0x00],
        [0x00, 0x00, 0xC6, 0x6C, 0x38, 0x6C, 0xC6, 0x00],
        [0x00, 0x00, 0xC6, 0xC6, 0xC6, 0x7E, 0x06, 0x7C],
        [0x00, 0x00, 0xFE, 0x0C, 0x38, 0x60, 0xFE, 0x00],
        [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00],
        [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00],
        [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00],
        [0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        [0x00, 0x10, 0x38, 0x6C, 0xC6, 0xC6, 0xFE, 0x00],
    ];

    let text_color = [255u8, 255, 255, 255];
    let char_h = 8u32;
    let char_w = 8u32;
    let start_x = (size - label.len() as u32 * char_w) / 2;
    let start_y = (size - char_h) / 2;

    for (ci, ch) in label.chars().enumerate() {
        let idx = ch as usize;
        if idx < 32 || idx > 126 {
            continue;
        }
        let bitmap = &font_8x8[idx - 32];
        let cx = start_x + ci as u32 * char_w;
        for row in 0..8 {
            let byte = bitmap[row];
            for col in 0..8 {
                if (byte >> (7 - col)) & 1 == 1 {
                    let px = cx + col as u32;
                    let py = start_y + row as u32;
                    if px < size && py < size {
                        let pidx = ((py * size + px) * 4) as usize;
                        pixels[pidx] = text_color[0];
                        pixels[pidx + 1] = text_color[1];
                        pixels[pidx + 2] = text_color[2];
                        pixels[pidx + 3] = text_color[3];
                    }
                }
            }
        }
    }

    pixels
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    texture_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    rotation: glam::Quat,
    distance: f32,
    mouse_pressed: bool,
    fps: u32,
    frame_count: u32,
    last_fps_instant: Instant,
}

impl<'a> State<'a> {
    async fn new(window: Arc<winit::window::Window>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (vertices, indices) = create_cube_vertices();
        let num_indices = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let face_colors: [[u8; 4]; 6] = [
            [220, 60, 60, 255],
            [60, 180, 60, 255],
            [60, 60, 220, 255],
            [220, 180, 40, 255],
            [200, 60, 200, 255],
            [60, 200, 200, 255],
        ];
        let face_labels = ["Front", "Back", "Right", "Left", "Top", "Bottom"];

        let atlas_width = 256 * 3;
        let atlas_height = 256 * 2;
        let mut atlas_pixels = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        for (fi, color) in face_colors.iter().enumerate() {
            let col = (fi % 3) as u32;
            let row = (fi / 3) as u32;
            let face_data = create_face_texture(*color, face_labels[fi]);
            for y in 0..256u32 {
                for x in 0..256u32 {
                    let src_idx = ((y * 256 + x) * 4) as usize;
                    let dst_x = col * 256 + x;
                    let dst_y = row * 256 + y;
                    let dst_idx = ((dst_y * atlas_width + dst_x) * 4) as usize;
                    atlas_pixels[dst_idx] = face_data[src_idx];
                    atlas_pixels[dst_idx + 1] = face_data[src_idx + 1];
                    atlas_pixels[dst_idx + 2] = face_data[src_idx + 2];
                    atlas_pixels[dst_idx + 3] = face_data[src_idx + 3];
                }
            }
        }

        let texture_size = wgpu::Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas_width),
                rows_per_image: Some(atlas_height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            texture_bind_group,
            uniform_buffer,
            uniform_bind_group,
            rotation: glam::Quat::IDENTITY,
            distance: 3.0,
            mouse_pressed: false,
            fps: 0,
            frame_count: 0,
            last_fps_instant: Instant::now(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_uniforms(&self) {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let fovy = 60.0f32.to_radians();
        let proj = glam::Mat4::perspective_rh(fovy, aspect, 0.1, 100.0);

        let eye = glam::Vec3::new(0.0, 0.0, self.distance);
        let target = glam::Vec3::ZERO;
        let up = glam::Vec3::Y;
        let view = glam::Mat4::look_at_rh(eye, target, up);

        let model = glam::Mat4::from_quat(self.rotation);
        let mvp = proj * view * model;

        let uniforms = Uniforms {
            model_view_proj: mvp.to_cols_array_2d(),
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_instant);
        if elapsed.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_instant = now;
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.12,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("3D Cube - 6 Photos")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = pollster::block_on(State::new(Arc::clone(&window)));

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::MouseInput {
                        state: btn_state,
                        button,
                        ..
                    } => {
                        if *button == MouseButton::Left {
                            let pressed = *btn_state == ElementState::Pressed;
                            state.mouse_pressed = pressed;
                            if pressed {
                                let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                                window.set_cursor_visible(false);
                            } else {
                                let _ = window.set_cursor_grab(CursorGrabMode::None);
                                window.set_cursor_visible(true);
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let dy = match delta {
                            MouseScrollDelta::LineDelta(_, y) => *y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                        };
                        state.distance = (state.distance - dy * 0.5).clamp(1.5, 10.0);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update_uniforms();
                        match state.render() {
                            Ok(_) => {
                                window
                                    .set_title(&format!("3D Cube - 6 Photos | FPS: {}", state.fps));
                            }
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => log::error!("Render error: {:?}", e),
                        }
                    }
                    _ => {}
                },
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if state.mouse_pressed {
                        let dx = delta.0 * MOUSE_SENSITIVITY;
                        let dy = -delta.1 * MOUSE_SENSITIVITY;
                        let angle = (dx * dx + dy * dy).sqrt() as f32;
                        if angle > 0.0001 {
                            let axis = glam::Vec3::new(-dy as f32, dx as f32, 0.0).normalize();
                            let rot = glam::Quat::from_axis_angle(axis, angle);
                            state.rotation = rot * state.rotation;
                        }
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
