/* -----------------------------------------------------------
    WGPU Tut 1
    Load an Wavefront Object file using tobj
   ----------------------------------------------------------- */
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
    _nor: [f32;3],
    _uv: [f32;2],
}
#[derive(Debug, Clone)]
struct Mesh {
    /*  since this is just a tryout to get the image on the screen, 
        I won't flesh out the rest of the struct.
        -- The obj file is broken up into groups of vertices
           that have a material (png) file associated with them.

    */ 


    vertices: Vec<Vertex>
}
// ----------------------------------------------------
fn get_model() -> Mesh {
    let root_path = format!(env!("CARGO_MANIFEST_DIR"));
    let asset_file = format!("{}/assets/house/house.obj", root_path);
    let (models, materials) = tobj::load_obj(&Path::new(&asset_file)).unwrap(); // will panic if not found! 
    let mut tvertices = Vec::new();
    for model in &models {
        let mesh = &model.mesh;
        for idx in &mesh.indices {  // The vertices are already indexed from the faces. 
                                    // this is needed to load all the vertices for the 
                                    // model.
            let j = *idx as usize;
            tvertices.push(Vertex {
                _pos: [mesh.positions[3*j],
                        mesh.positions[3*j+1],
                        mesh.positions[3*j+2]],
                _nor: { if !mesh.normals.is_empty() {
                        [mesh.normals[3*j],
                        mesh.normals[3*j+1],
                        mesh.normals[3*j+2]]
                        } else { [0.0,0.0,0.0]}},
                _uv: [mesh.texcoords[2*j],
                        mesh.texcoords[2*j+1]],
            });
        }
    }

    let mesh = Mesh { vertices: tvertices };
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
// Setup the "factory"
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
// Load the vertex buffer - it's already indexed! 
    let vertex_size = std::mem::size_of::<Vertex>();
    let vertex_buf = device
        .create_buffer_mapped(mesh.vertices.len(), wgpu::BufferUsageFlags::VERTEX)
        .fill_from_slice(&mesh.vertices);
    let vertex_count = mesh.vertices.len();

//  no textures yet - just want to see the triangles
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
            front_face: wgpu::FrontFace::Cw,
            cull_mode: wgpu::CullMode::Back,
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
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: &[wgpu::VertexBufferDescriptor {
            stride: vertex_size as u32,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {   // _pos
                    attribute_index: 0,
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                },
                wgpu::VertexAttributeDescriptor {   // _nor
                    attribute_index: 1,
                    format: wgpu::VertexFormat::Float3,
                    offset: 4 * 3,                  // size f32 * # of elements
                },                
                wgpu::VertexAttributeDescriptor {   // _uv
                    attribute_index: 2,
                    format: wgpu::VertexFormat::Float2,
                    offset: 4 * (3 + 3),            // offset from the first element -
                                                    // 2 Float3 = 6 
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
//            rpass.set_index_buffer(&index_buf, 0); // not needed
            rpass.set_vertex_buffers(&[(&vertex_buf, 0)]);
            rpass.draw(0 .. vertex_count as u32, 0 .. 1);       // just plain draw - already indexed
        }
        device.get_queue().submit(&[encoder.finish()]);
    }
    println!("Exiting");
}

