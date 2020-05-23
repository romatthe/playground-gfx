use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{WindowBuilder, Window};
use winit::event::{Event, WindowEvent, ElementState, KeyboardInput, VirtualKeyCode};
use winit::dpi::PhysicalSize;
use wgpu::{BackendBit, DeviceDescriptor, SwapChainDescriptor, TextureFormat, Adapter, Surface, Device, Queue, SwapChain, TextureUsage, PresentMode, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachmentDescriptor, LoadOp, StoreOp, Color};
use futures::executor;

struct State {
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    size: PhysicalSize<u32>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let surface = wgpu::Surface::create(window);

        let adapter = Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            // Vulkan + Metal + DX12 + Browser WebGPU
            BackendBit::PRIMARY,
        ).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false
            },
            limits: Default::default()
        }).await;

        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            size
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

    fn update(&mut self) {

    }

    fn render(&mut self) {
        let frame = self.swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[
                    RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: LoadOp::Clear,
                        store_op: StoreOp::Clear,
                        clear_color: Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0
                        }
                    }
                ],
                depth_stencil_attachment: None
            });
        }

        self.queue.submit(&[
            encoder.finish()
        ]);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    // Since main can't be async, we're going to need to block
    let mut state = executor::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { ref event, window_id} if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        match input {
                            KeyboardInput {
                               state: ElementState::Pressed,
                               virtual_keycode: Some(VirtualKeyCode::Escape),
                               ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => ()
                        }
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.resize(**new_inner_size)
                    }
                    _ => ()
                }
            },
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it
                window.request_redraw();
            },
            _ => ()
        }
    });
}