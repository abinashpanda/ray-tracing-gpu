use shared::ShaderConstants;
use winit::{
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

unsafe fn any_as_u32_slice<T: Sized>(p: &T) -> &[u32] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u32,
        ::std::mem::size_of::<T>() / 4,
    )
}

async fn run(
    shader_module: wgpu::ShaderModuleSource<'static>,
    event_loop: EventLoop<()>,
    window: Window,
    swapchain_format: wgpu::TextureFormat,
) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let mut surface = Some(unsafe { instance.create_surface(&window) });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            // Request an adapter which can render to our surface
            compatible_surface: surface.as_ref(),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let features = wgpu::Features::PUSH_CONSTANTS;
    let limits = wgpu::Limits {
        max_push_constant_size: 256,
        ..Default::default()
    };

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features,
                limits,
                shader_validation: true,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let module = device.create_shader_module(shader_module);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStage::all(),
            range: 0..std::mem::size_of::<ShaderConstants>() as u32,
        }],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &module,
            entry_point: "main_vs",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &module,
            entry_point: "main_fs",
        }),
        // Use the default rasterizer state: no culling, no depth bias
        rasterization_state: None,
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[swapchain_format.into()],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = surface
        .as_ref()
        .map(|surface| device.create_swap_chain(&surface, &sc_desc));

    let start = std::time::Instant::now();
    let (mut cursor_x, mut cursor_y) = (0.0, 0.0);
    let (mut drag_start_x, mut drag_start_y) = (0.0, 0.0);
    let (mut drag_end_x, mut drag_end_y) = (0.0, 0.0);
    let mut mouse_left_pressed = false;
    let mut mouse_left_clicked = false;

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &module, &pipeline_layout);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::Resumed => {
                let s = unsafe { instance.create_surface(&window) };
                swap_chain = Some(device.create_swap_chain(&s, &sc_desc));
                surface = Some(s);
            }
            Event::Suspended => {
                surface = None;
                swap_chain = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                if let Some(surface) = &surface {
                    swap_chain = Some(device.create_swap_chain(surface, &sc_desc));
                }
            }
            Event::RedrawRequested(_) => {
                if let Some(swap_chain) = &mut swap_chain {
                    let frame = swap_chain
                        .get_current_frame()
                        .expect("Failed to acquire next swap chain texture")
                        .output;
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: true,
                                },
                            }],
                            depth_stencil_attachment: None,
                        });
                        let push_constants = ShaderConstants {
                            width: window.inner_size().width,
                            height: window.inner_size().height,
                            time: start.elapsed().as_secs_f32(),
                            cursor_x,
                            cursor_y,
                            drag_start_x,
                            drag_start_y,
                            drag_end_x,
                            drag_end_y,
                            mouse_left_pressed,
                            mouse_left_clicked,
                        };
                        mouse_left_clicked = false;
                        rpass.set_pipeline(&render_pipeline);
                        rpass.set_push_constants(wgpu::ShaderStage::all(), 0, unsafe {
                            any_as_u32_slice(&push_constants)
                        });
                        rpass.draw(0..3, 0..1);
                    }

                    queue.submit(Some(encoder.finish()));
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                mouse_left_pressed = state == ElementState::Pressed;
                if mouse_left_pressed {
                    drag_start_x = cursor_x;
                    drag_start_y = cursor_y;
                    drag_end_x = cursor_x;
                    drag_end_y = cursor_y;
                    mouse_left_clicked = true;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                cursor_x = position.x as f32;
                cursor_y = position.y as f32;
                if mouse_left_pressed {
                    drag_end_x = cursor_x;
                    drag_end_y = cursor_y;
                }
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Rust GPU - wgpu")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    wgpu_subscriber::initialize_default_subscriber(None);
    futures::executor::block_on(run(
        wgpu::include_spirv!(env!("ray_tracing_shaders.spv")),
        event_loop,
        window,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    ));
}
