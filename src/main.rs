extern crate wgpu;
extern crate cgmath;
extern crate tobj;
extern crate glsl_to_spirv;

use wgpu::winit;
use std::path::{Path, PathBuf};
use std::fs::{read_to_string};
use std::io::{Read};
use std::mem;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    _pos: [f32;3],
    _uv: [f32;2],
    _nor: [f32;3],

}
#[derive(Debug, Clone)]
struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
// ----------------------------------------------------
fn get_model() -> Mesh {
    let root_path = format!(env!("CARGO_MANIFEST_DIR"));
    let asset_file = format!("{}/assets/house/house.obj", root_path);
    let (model, materials) = tobj::load_obj(&Path::new(&asset_file)).unwrap();
    let tmesh = &model[0].mesh;
    let mesh_size = tmesh.texcoords.len() /2;
    let mut tvertices = Vec::new();
    for i in 0..mesh_size {
        tvertices.push( Vertex {
            _pos: [tmesh.positions[i*3], tmesh.positions[i*3+1], tmesh.positions[i*3+2]],
            _uv: [tmesh.texcoords[i*2], tmesh.texcoords[i*2+1]],
            _nor: [tmesh.normals[i*3], tmesh.normals[i*3+1], tmesh.normals[i*3+2]],
        });
    };
    let tindices = tmesh.indices.clone();
    let mesh = Mesh { vertices: tvertices, indices: tindices, };
    mesh
}
#[allow(dead_code)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

fn load_glsl(name: &str, stage: ShaderStage) -> Vec<u8> {
        let ty = match stage {
        ShaderStage::Vertex => glsl_to_spirv::ShaderType::Vertex,
        ShaderStage::Fragment => glsl_to_spirv::ShaderType::Fragment,
        ShaderStage::Compute => glsl_to_spirv::ShaderType::Compute,
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/shader")
        .join(name);
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };

    let mut output = glsl_to_spirv::compile(&code, ty).unwrap();
    let mut spv = Vec::new();
    output.read_to_end(&mut spv).unwrap();
    spv
}

// ----------------------------------------------------
fn main()   {
    println!("Hello, world!");
// Create the main window...
    let mut event_loop = wgpu::winit::EventsLoop::new();
    let mut width = 800.0;
    let mut height = 640.0;
    let ( _window, instance, size, surface) = {
        let instance = wgpu::Instance::new();
        let _window = winit::WindowBuilder::new()
            .with_title("WGPU One")
            .with_dimensions(winit::dpi::LogicalSize {
                width: width as f64, height: height as f64,})
            .build(&event_loop)
            .unwrap();
        let size = _window
            .get_inner_size()
            .unwrap()
            .to_physical(_window.get_hidpi_factor());
        let surface = instance.create_surface(&_window);
        ( _window, instance, size, surface)
    };

    let mesh = get_model();
// ----------------------------------------------------
// Setup Hardware
    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::HighPerformance,
    });
    let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        }
    });
// Load shaders
    let vs_bytes = load_glsl("triangle.vert", ShaderStage::Vertex);
    let fs_bytes = load_glsl("triangle.frag", ShaderStage::Fragment);
    let vs_module = device.create_shader_module(&vs_bytes);
    let fs_module = device.create_shader_module(&fs_bytes);
    println!("After shader load");
// Prep Buffers
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_buf = device
        .create_buffer_mapped(mesh.vertices.len(), wgpu::BufferUsageFlags::VERTEX)
        .fill_from_slice(&mesh.vertices);
    let index_buf = device
        .create_buffer_mapped(mesh.indices.len(), wgpu::BufferUsageFlags::INDEX)
        .fill_from_slice(&mesh.indices);
    let index_count = mesh.indices.len();
    let vertex_count = mesh.vertices.len();

    let bind_group_layout = 
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[],
        });
    let bind_group = 
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[],
        });
// Setup Pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        },
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Front,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color: wgpu::BlendDescriptor::REPLACE,
            alpha: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWriteFlags::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: vertex_size as u32,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 0,
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                },

                wgpu::VertexAttributeDescriptor {
                    attribute_index: 1,
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * 3,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 2,
                    format: wgpu::VertexFormat::Float3,
                    offset: 4 * (3 + 2),
                },
            ]
        }],
        sample_count: 1,
    });

// Swap chain
    let mut swap_chain = device.create_swap_chain(
        &surface, 
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width.round() as u32,
            height: size.height.round() as u32,
        },
    );

// ----------------------------------------------------
    let clear_color = wgpu::Color { r: 0.0, g: 0.5, b: 0.5, a: 1.0 };
    let mut running = true;
    while running {
        event_loop.poll_events(|event| { match event {
            winit::Event::WindowEvent { event, .. } => match event {
                winit::WindowEvent::CloseRequested => running = false,
                _ => (),
            },
// other event processing -- keyboard,mouse
            _ => (),
            }
        });

// Draw stuff here ... 
        let frame = swap_chain.get_next_texture();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            todo: 0,
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: clear_color,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group);
            rpass.set_index_buffer(&index_buf, 0);
            rpass.set_vertex_buffers(&[(&vertex_buf, 0)]);
            rpass.draw_indexed(0 .. index_count as u32, 0, 0 .. 1);
        }
        device.get_queue().submit(&[encoder.finish()]);
    }
    println!("Exiting");
}

