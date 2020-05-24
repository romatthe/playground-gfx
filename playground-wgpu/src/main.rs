mod texture;

use futures::executor;
use std::mem;
use wgpu::{Adapter, BackendBit, BlendDescriptor, Buffer, BufferAddress, BufferUsage, Color, ColorStateDescriptor, ColorWrite, CommandEncoderDescriptor, CullMode, Device, DeviceDescriptor, FrontFace, IndexFormat, InputStepMode, LoadOp, PipelineLayoutDescriptor, PresentMode, PrimitiveTopology, ProgrammableStageDescriptor, Queue, RasterizationStateDescriptor, RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StoreOp, Surface, SwapChain, SwapChainDescriptor, TextureFormat, TextureUsage, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat, VertexStateDescriptor, Extent3d, TextureDescriptor, TextureDimension, BufferCopyView, TextureCopyView, Origin3d, SamplerDescriptor, AddressMode, FilterMode, CompareFunction, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, TextureViewDimension, TextureComponentType, ShaderStage, BindGroupDescriptor, Binding, BindingResource, Texture, TextureView, Sampler, BindGroup};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use image::GenericImageView;

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397057], },
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732911], },
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn descriptor<'a>() -> VertexBufferDescriptor<'a> {
        VertexBufferDescriptor {
            // How wide is the Vertex
            stride: mem::size_of::<Vertex>() as BufferAddress,
            // How often should it move to the next vertex
            step_mode: InputStepMode::Vertex,
            // Attributes of our vertex data
            attributes: &[
                VertexAttributeDescriptor {
                    // Where does the attribute start?
                    offset: 0,
                    // Where to store the attribute, ex: layout(location=0) in vec3 x would be position
                    shader_location: 0,
                    // Shape of the attribute, corresponds to vec3 in shader
                    format: VertexFormat::Float3,
                },
                VertexAttributeDescriptor {
                    // Where does the attribute start?
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    // Where to store the attribute, ex: layout(location=1) in vec3 x would be color
                    shader_location: 1,
                    // Shape of the attribute, corresponds to vec3 in shader
                    format: VertexFormat::Float2,
                },
            ],
        }
    }
}

// Plain old data: Can be interpreted as &[u8]
unsafe impl bytemuck::Pod for Vertex {}

// We can use std::mem::zeroed()
unsafe impl bytemuck::Zeroable for Vertex {}

struct State {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,

    // Texture
    diffuse_texture: texture::Texture,
    diffuse_bind_group: BindGroup,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);
        let num_indices = INDICES.len() as u32;

        let adapter = Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            // Vulkan + Metal + DX12 + Browser WebGPU
            BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, mut queue) = adapter
            .request_device(&DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Load the tree picture
        let diffuse_bytes = include_bytes!("../resources/happy-tree.png");
        let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(&device, diffuse_bytes).unwrap();

        queue.submit(&[cmd_buffer]);

        // let diffuse_texture_view = diffuse_texture.create_default_view();
        // let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
        //     address_mode_u: AddressMode::ClampToEdge,
        //     address_mode_v: AddressMode::ClampToEdge,
        //     address_mode_w: AddressMode::ClampToEdge,
        //     mag_filter: FilterMode::Linear,
        //     min_filter: FilterMode::Nearest,
        //     mipmap_filter: FilterMode::Nearest,
        //     lod_min_clamp: -100.0,
        //     lod_max_clamp: 100.0,
        //     compare: CompareFunction::Always
        // });
        //
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                    },
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // Include GLSL shaders
        let vs_src = include_str!("../shaders/shader.vert");
        let fs_src = include_str!("../shaders/shader.frag");

        // Compile the shaders
        let vs_spirv = glsl_to_spirv::compile(vs_src, glsl_to_spirv::ShaderType::Vertex).unwrap();
        let fs_spirv = glsl_to_spirv::compile(fs_src, glsl_to_spirv::ShaderType::Fragment).unwrap();

        // Load the SPIR-V data
        let vs_data = wgpu::read_spirv(vs_spirv).unwrap();
        let fs_data = wgpu::read_spirv(fs_spirv).unwrap();

        // Create shader modules
        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&texture_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            // describes how to process primitives before they are sent to the fragment shader
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            // Describes how colors are stored and processed throughout the pipeline
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                alpha_blend: BlendDescriptor::REPLACE,
                color_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            // We're drawing a list of triangles
            primitive_topology: PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: VertexStateDescriptor {
                // Use 16-bit integers for indexing
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[Vertex::descriptor()],
            },
            sample_count: 1,
            // Specifies which samples should be active, !0 is all of them
            sample_mask: !0,
            // No anti-aliasing
            alpha_to_coverage_enabled: false,
        });

        let vertex_buffer =
            device.create_buffer_with_data(bytemuck::cast_slice(VERTICES), BufferUsage::VERTEX);

        let index_buffer =
            device.create_buffer_with_data(bytemuck::cast_slice(INDICES), BufferUsage::INDEX);

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            diffuse_bind_group,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&self.index_buffer, 0, 0);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Since main can't be async, we're going to need to block
    let mut state = executor::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => (),
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size)
                        }
                        _ => (),
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it
                window.request_redraw();
            }
            _ => (),
        }
    });
}
