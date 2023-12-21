use egui_wgpu::{renderer::ScreenDescriptor, Renderer};
use egui_winit::State;
use std::borrow::Cow;
use std::iter;
use wgpu;
use wgpu::InstanceDescriptor;
use winit::{
    event::{Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

const WGSL_SHADERS: &str = "
struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex_main(vert: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = vert.color;
    out.position = vert.position;
    return out;
};

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
";

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut window_size = window.inner_size();
    window_size.width = window_size.width.max(1);
    window_size.height = window_size.height.max(1);

    //let instance_descriptor = InstanceDescriptor::default();
    //let instance = wgpu::Instance::new(instance_descriptor);
    let instance = wgpu::Instance::default();
    let surface = unsafe {
        instance
            .create_surface(&window)
            .expect("Error - could not get a surface")
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find a WebGPU adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let capabilities = surface.get_capabilities(&adapter);
    //let surface_format = *capabilities.formats.iter().find(|f| f.is_srgb()).expect("Could not find srgb surface");

    // Chrome Canary complains about unknown swapchain format
    #[cfg(target_arch = "wasm32")]
    let surface_format = wgpu::TextureFormat::Bgra8Unorm;
    #[cfg(not(target_arch = "wasm32"))]
    let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    let context = egui::Context::default();
    context.set_style(egui::Style::default());
    context.set_pixels_per_point(window.scale_factor() as f32);

    let mut state = State::new(context.viewport_id(), &window, None, None);
    //state.set_pixels_per_point(window.scale_factor() as f32);

    //let mut state = egui_winit::State::new(&event_loop);
    //let context = egui::Context::default();
    //context.set_pixels_per_point(window.scale_factor() as f32);

    let mut egui_rpass = Renderer::new(&device, surface_format, None, 1);

    let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SHADERS)),
    });
    let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SHADERS)),
    });

    let vertex_data: [f32; 24] = [
        1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0,
        1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0,
    ];
    let data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (vertex_data.len() * 4) as u64,
        usage: wgpu::BufferUsages::VERTEX,
        mapped_at_creation: true,
    });
    {
        let mut view = data_buffer.slice(..).get_mapped_range_mut();
        let float_view = unsafe {
            std::slice::from_raw_parts_mut(view.as_mut_ptr() as *mut f32, vertex_data.len())
        };
        float_view.copy_from_slice(&vertex_data)
    }
    data_buffer.unmap();

    let index_data: [u16; 3] = [0, 1, 2];
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (index_data.len() * 4) as u64,
        usage: wgpu::BufferUsages::INDEX,
        mapped_at_creation: true,
    });
    {
        let mut view = index_buffer.slice(..).get_mapped_range_mut();
        let u16_view = unsafe {
            std::slice::from_raw_parts_mut(view.as_mut_ptr() as *mut u16, index_data.len())
        };
        u16_view.copy_from_slice(&index_data)
    }
    index_buffer.unmap();

    let vertex_attrib_descs = [
        wgpu::VertexAttribute {
            offset: 0,
            format: wgpu::VertexFormat::Float32x4,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            offset: 4 * 4,
            format: wgpu::VertexFormat::Float32x4,
            shader_location: 1,
        },
    ];

    let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
        array_stride: 2 * 4 * 4,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attrib_descs,
    }];

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_module,
            entry_point: "vertex_main",
            buffers: &vertex_buffer_layouts,
        },
        primitive: wgpu::PrimitiveState {
            // Note: it's not possible to set a "none" strip index format,
            // which raises an error in Chrome Canary b/c when using non-strip
            // topologies, the index format must be none. However, wgpu-rs
            // instead defaults this to uint16, leading to an invalid state.
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: Some(wgpu::IndexFormat::Uint16),
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
            unclipped_depth: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_module,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });

    event_loop.run(move |event, _, control_flow| {
        //*control_flow = ControlFlow::Poll;

        if let Event::WindowEvent { event, .. } = &event {
            let response = state.on_window_event(&context, event);
            if response.repaint {
                window.request_redraw();
            }
            if response.consumed {
                return;
            }
        }

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. }
                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) =>
                {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        window_size.width = size.width;
                        window_size.height = size.height;
                        println!("scale factor: {:?}", window.scale_factor());
                        println!("w/h: {:?} {:?}", window_size.width, window_size.height);
                        println!(
                            "used size: {:?} {:?}",
                            context.used_size().x,
                            context.used_size().y
                        );
                        surface.configure(
                            &device,
                            &wgpu::SurfaceConfiguration {
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                format: surface_format,
                                width: window_size.width,
                                height: window_size.height,
                                present_mode: wgpu::PresentMode::Fifo,
                                alpha_mode: capabilities.alpha_modes[0],
                                view_formats: vec![],
                            },
                        );
                    }
                }
                WindowEvent::DroppedFile(file) => {
                    println!("File dropped: {:?}", file.as_path().display().to_string());
                    // do model loading here
                }
                WindowEvent::CursorMoved { position, .. } => {
                    println!("pos_x: {:?} pos_y: {:?}", position.x, position.y);
                    /*if context.is_pointer_over_area()*/
                    {
                        state.on_window_event(&context, &event);
                    }
                }
                _ => {
                    // forward events to egui
                    state.on_window_event(&context, &event);
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        // "The underlying surface has changed, and therefore the swap chain must be updated"
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let input = state.take_egui_input(&window);

                context.begin_frame(input);

                egui::Window::new("K4 Kahlberg").show(&context, |ui| {
                    ui.heading("Objects");
                    ui.label("Currently there are no objects here.");
                    ui.separator();
                    ui.heading("Settings");
                    ui.label("Show fog");
                    ui.label("Show crosshair");
                    ui.label("See https://github.com/emilk/egui for how to make other UI elements");
                    if ui.button("Switch to light mode").clicked() {
                        //egui::widgets::global_dark_light_mode_switch(ui);
                        context.set_visuals(egui::Visuals::light());
                    }
                });
                let output = context.end_frame();

                let paint_jobs = context.tessellate(output.shapes, context.pixels_per_point());

                state.handle_platform_output(&window, &context, output.platform_output);

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor {
                    size_in_pixels: [window_size.width, window_size.height],
                    pixels_per_point: window.scale_factor() as f32,
                };

                let tdelta: egui::TexturesDelta = output.textures_delta;
                for (tid, deltas) in tdelta.set {
                    egui_rpass.update_texture(&device, &queue, tid, &deltas);
                }
                egui_rpass.update_buffers(
                    &device,
                    &queue,
                    &mut encoder,
                    &paint_jobs,
                    &screen_descriptor,
                );

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &output_view,
                            resolve_target: None,
                            ops: Default::default(),
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_vertex_buffer(0, data_buffer.slice(..));
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Note: also bug in wgpu-rs set_index_buffer or web sys not passing
                        // the right index type
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..3, 0, 0..1);
                    }
                    // This is actually kind of wrong to do, but it kind of works out anyways
                    #[cfg(target_arch = "wasm32")]
                    {
                        render_pass.draw(0..3, 0..1);
                    }

                    egui_rpass.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                }
                queue.submit(iter::once(encoder.finish()));
                output_frame.present();

                for tid in tdelta.free {
                    egui_rpass.free_texture(&tid);
                }
            }
            _ => (),
        }
    });
}

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::<()>::with_user_event().build();
    #[allow(unused_mut)]
    let mut builder = winit::window::WindowBuilder::new()
        .with_title("K4 Kahlberg")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        });

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run(event_loop, window));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .expect("error window")
            .document()
            .expect("error document")
            .get_element_by_id("canvas")
            .expect("error anchor")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("error HtmlCanvasElement");
        builder = builder.with_canvas(Some(canvas));
    }

    let window = builder
        .build(&event_loop)
        .expect("Could not create Event Loop!");

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
