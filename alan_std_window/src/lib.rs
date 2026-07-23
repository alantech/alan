/// Alan module: @std/window
/// Provides window and GPU rendering primitives.
///
/// Import with:
///   type Window <-- '@std/window'
///   type Frame <-- '@std/window'
///   fn GBuffer <-- '@std/window'
///   fn window <-- '@std/window'

/// Marker type for window-scope Rust bindings (distinct from root-scope RootBacking).
pub struct WindowBacking;

// Re-export GBuffer and GPGPU types from alan_std so window code can use them
pub use alan_std::buffer_id;
pub use alan_std::bufferlen;
pub use alan_std::create_buffer_init;
pub use alan_std::create_empty_buffer;
pub use alan_std::gpu_run;
pub use alan_std::gpu_run_list;
pub use alan_std::map_read_buffer_type;
pub use alan_std::map_write_buffer_type;
pub use alan_std::optimal_local_group;
pub use alan_std::read_buffer;
pub use alan_std::replace_buffer;
pub use alan_std::storage_buffer_type;
pub use alan_std::AlanError;
pub use alan_std::GBuffer;
pub use alan_std::GPGPU;

/// Window context struct -- holds window state and input tracking.
#[derive(Clone)]
pub struct AlanWindowContext {
    pub window: Option<std::sync::Arc<winit::window::Window>>,
    pub start: Option<std::time::Instant>,
    pub buffer_width: Option<u32>,
    pub mouse_x: Option<u32>,
    pub mouse_y: Option<u32>,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub mouse_middle: bool,
    pub mouse_wheel_dx: f32,
    pub mouse_wheel_dy: f32,
    pub cursor_visible: bool,
    pub transparent: bool,
}

impl AlanWindowContext {
    pub fn width(&self) -> u32 {
        match self.window.as_ref() {
            Some(win) => win.inner_size().width.max(1),
            None => 0,
        }
    }

    pub fn height(&self) -> u32 {
        match self.window.as_ref() {
            Some(win) => win.inner_size().height.max(1),
            None => 0,
        }
    }

    pub fn buffer_width(&self) -> u32 {
        self.buffer_width.unwrap_or(0) / 4
    }

    pub fn runtime(&self) -> u32 {
        match self.start.as_ref() {
            Some(time) => u32::from_le_bytes(time.elapsed().as_secs_f32().to_le_bytes()),
            None => 0,
        }
    }

    pub fn mouse_x(&mut self) -> u32 {
        match self.mouse_x {
            Some(x) => x,
            None => {
                self.mouse_x = Some(0);
                self.mouse_y = Some(0);
                0
            }
        }
    }

    pub fn mouse_y(&mut self) -> u32 {
        match self.mouse_y {
            Some(y) => y,
            None => {
                self.mouse_x = Some(0);
                self.mouse_y = Some(0);
                0
            }
        }
    }

    pub fn cursor_visible(&mut self) {
        self.cursor_visible = true;
    }

    pub fn cursor_invisible(&mut self) {
        self.cursor_visible = false;
    }

    pub fn transparent(&mut self) {
        self.transparent = true;
    }

    pub fn opaque(&mut self) {
        self.transparent = false;
    }

    pub fn mouse_left(&mut self) -> u32 {
        self.mouse_left as u32
    }

    pub fn mouse_right(&mut self) -> u32 {
        self.mouse_right as u32
    }

    pub fn mouse_middle(&mut self) -> u32 {
        self.mouse_middle as u32
    }

    pub fn mouse_wheel_x(&mut self) -> f32 {
        let v = self.mouse_wheel_dx;
        self.mouse_wheel_dx = 0.0;
        v
    }

    pub fn mouse_wheel_y(&mut self) -> f32 {
        let v = self.mouse_wheel_dy;
        self.mouse_wheel_dy = 0.0;
        v
    }
}

/// Frame struct passed to the GPU shader function.
pub struct AlanWindowFrame {
    pub context: GBuffer,
    pub framebuffer: GBuffer,
    pub width: u32,
    pub height: u32,
}

// Type aliases for window function signatures
type WindowContextFn = Box<dyn FnMut(&mut AlanWindowContext) -> Vec<u32>>;
type WindowGPGPUShaderFn = Box<dyn Fn(&AlanWindowFrame) -> Vec<GPGPU>>;

/// Per-window GPU and rendering state
struct WindowState {
    context: AlanWindowContext,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    context_buffer: Option<GBuffer>,
    buffer: Option<GBuffer>,
    cached_surface_config: Option<wgpu::SurfaceConfiguration>,
    cached_size: winit::dpi::PhysicalSize<u32>,
    context_fn: WindowContextFn,
    gpgpu_shader_fn: WindowGPGPUShaderFn,
    gpgpu_shaders: Option<Vec<GPGPU>>,
    inited: bool,
}

/// Manages all open windows in the event loop
struct WindowManager {
    window: Option<std::sync::Arc<winit::window::Window>>,
    state: Option<WindowState>,
    init: Option<WindowInit>,
}

impl WindowManager {
    fn gpu_init(&mut self) {
        let ws = self.state.as_mut().unwrap();
        let window = self.window.as_ref().unwrap().clone();
        ws.context.window = Some(window);
        if ws.context.start.is_none() {
            ws.context.start = Some(std::time::Instant::now());
        }
        if ws.surface.is_none() {
            ws.surface = Some(alan_std::instance().create_surface(self.window.as_ref().unwrap().clone()).unwrap());
        }
        if ws.device.is_none() {
            let g = alan_std::gpu();
            ws.device = Some(g.get_device().clone());
            ws.queue = Some(g.get_queue().clone());
        }
        if ws.context_buffer.is_none() {
            ws.context_buffer = Some(create_empty_buffer(&storage_buffer_type(), &64, &4).unwrap());
        }
        if ws.buffer.is_none() {
            let mut size = self.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            ws.context.buffer_width = Some(if (4 * size.width).is_multiple_of(256) {
                4 * size.width
            } else {
                (4 * size.width) + (256 - ((4 * size.width) % 256))
            });
            let buffer_size = (ws.context.buffer_width.unwrap() as u64) * (size.height as u64);
            ws.buffer = Some(
                create_empty_buffer(&storage_buffer_type(), &(buffer_size as i64), &4).unwrap(),
            );
        }
        if ws.gpgpu_shaders.is_none() {
            let mut size = self.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            ws.gpgpu_shaders = Some((ws.gpgpu_shader_fn)(&AlanWindowFrame {
                context: ws.context_buffer.as_ref().unwrap().clone(),
                framebuffer: ws.buffer.as_ref().unwrap().clone(),
                width: size.width,
                height: size.height,
            }));
        }
        ws.inited = true;
    }

    fn render_frame(&mut self) {
        let ws = self.state.as_mut().unwrap();
        if !ws.inited {
            self.gpu_init();
        }
        let ws = self.state.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();
        window.set_cursor_visible(ws.context.cursor_visible);
        window.set_transparent(ws.context.transparent);
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let surface = ws.surface.as_ref().unwrap();
        let g = alan_std::gpu();
        let device = ws.device.as_ref().unwrap();
        let queue = ws.queue.as_ref().unwrap();
        if ws.cached_surface_config.is_none() || ws.cached_size != size {
            let mut config = surface.get_default_config(&g.adapter, size.width, size.height).unwrap();
            config.usage = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            config.desired_maximum_frame_latency = 1;
            config.alpha_mode = if ws.context.transparent {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };
            surface.configure(device, &config);
            ws.cached_surface_config = Some(config);
            ws.cached_size = size;
        }
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return,
        };
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_array = (ws.context_fn)(&mut ws.context);
        let context_slice = &context_array[..];
        let context_ptr = context_slice.as_ptr();
        let context_u8_len = context_array.len() * 4;
        let context_u8: &[u8] =
            unsafe { std::slice::from_raw_parts(context_ptr as *const u8, context_u8_len) };
        let ctx_buf = ws.context_buffer.as_ref().unwrap();
        queue.write_buffer(&**ctx_buf, 0, context_u8);
        let ggs = ws.gpgpu_shaders.as_mut().unwrap();
        for gg in ggs {
            if gg.module.is_none() {
                gg.module = Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
                }));
            }
            let module = gg.module.as_ref().unwrap();
            if gg.compute_pipeline.is_none() {
                gg.compute_pipeline = Some(device.create_compute_pipeline(
                    &wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: None,
                        module,
                        entry_point: Some(&gg.entrypoint),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    },
                ));
            }
            let compute_pipeline = gg.compute_pipeline.as_ref().unwrap();
            let mut bind_groups = Vec::new();
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                cpass.set_pipeline(compute_pipeline);
                for i in 0..gg.buffers.len() {
                    let bind_group_layout =
                        compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
                    let bind_group_buffers = &gg.buffers[i];
                    let mut bind_group_entries = Vec::new();
                    for j in 0..bind_group_buffers.len() {
                        bind_group_entries.push(wgpu::BindGroupEntry {
                            binding: j.try_into().unwrap(),
                            resource: bind_group_buffers[j].as_entire_binding(),
                        });
                    }
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &bind_group_layout,
                        entries: &bind_group_entries[..],
                    });
                    bind_groups.push(bind_group);
                }
                for i in 0..gg.buffers.len() {
                    cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
                }
                let lx = gg.local_workgroup_size[0];
                let ly = gg.local_workgroup_size[1];
                cpass.dispatch_workgroups(
                    ((gg.workgroup_sizes[0] + lx - 1) / lx) as u32,
                    ((gg.workgroup_sizes[1] + ly - 1) / ly) as u32,
                    gg.workgroup_sizes[2] as u32,
                );
            }
        }
        let framebuffer = ws.buffer.as_ref().unwrap();
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &**framebuffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: ws.context.buffer_width,
                    rows_per_image: None,
                },
            },
            frame.texture.as_image_copy(),
            frame.texture.size(),
        );
        queue.submit(Some(encoder.finish()));
        queue.present(frame);
        let frame_start = std::time::Instant::now();
        let render_time = frame_start.elapsed();
        window.set_title(&format!("Render time: {:.3}", render_time.as_secs_f64()));
        window.request_redraw();
    }
}

struct WindowInit {
    config: winit::window::WindowAttributes,
    context: AlanWindowContext,
    context_fn: WindowContextFn,
    gpgpu_shader_fn: WindowGPGPUShaderFn,
}

impl winit::application::ApplicationHandler<()> for WindowManager {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
            return;
        }
        let init = self.init.take().unwrap();
        let window = std::sync::Arc::new(event_loop.create_window(init.config).unwrap());
        let mut context = init.context;
        context.window = Some(window.clone());
        self.window = Some(window);
        self.state = Some(WindowState {
            context,
            surface: None,
            device: None,
            queue: None,
            context_buffer: None,
            buffer: None,
            cached_surface_config: None,
            cached_size: winit::dpi::PhysicalSize::new(0, 0),
            context_fn: init.context_fn,
            gpgpu_shader_fn: init.gpgpu_shader_fn,
            gpgpu_shaders: None,
            inited: false,
        });
        self.window.as_ref().unwrap().request_redraw();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                if let Some(ws) = self.state.take() {
                    if let Some(b) = &ws.buffer {
                        b.destroy();
                    }
                    if let Some(b) = &ws.context_buffer {
                        b.destroy();
                    }
                }
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(mut new_size) => {
                if event_loop.exiting() {
                    return;
                }
                let ws = match self.state.as_ref() {
                    Some(ws) => ws,
                    None => return,
                };
                if !ws.inited {
                    return;
                }
                new_size.width = new_size.width.max(1);
                new_size.height = new_size.height.max(1);
                let buffer_width = if (4 * new_size.width) % 256 == 0 {
                    4 * new_size.width
                } else {
                    (4 * new_size.width) + (256 - ((4 * new_size.width) % 256))
                };
                let buffer_size = (buffer_width as u64) * (new_size.height as u64);
                let new_buffer =
                    create_empty_buffer(&storage_buffer_type(), &(buffer_size as i64), &4).unwrap();
                let ws = self.state.as_mut().unwrap();
                if let Some(b) = &ws.buffer {
                    b.destroy();
                }
                ws.buffer = Some(new_buffer);
                ws.context.buffer_width = Some(buffer_width);
                ws.gpgpu_shaders = Some((ws.gpgpu_shader_fn)(&AlanWindowFrame {
                    context: ws.context_buffer.as_ref().unwrap().clone(),
                    framebuffer: ws.buffer.as_ref().unwrap().clone(),
                    width: new_size.width,
                    height: new_size.height,
                }));
                self.window.as_ref().unwrap().request_redraw();
            }
            winit::event::WindowEvent::RedrawRequested => {
                if event_loop.exiting() {
                    return;
                }
                self.render_frame();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                if let Some(ws) = self.state.as_mut() {
                    if ws.context.mouse_x.is_some() {
                        ws.context.mouse_x = Some(position.x as u32);
                        ws.context.mouse_y = Some(position.y as u32);
                    }
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == winit::event::ElementState::Pressed;
                if let Some(ws) = self.state.as_mut() {
                    match button {
                        winit::event::MouseButton::Left => ws.context.mouse_left = pressed,
                        winit::event::MouseButton::Right => ws.context.mouse_right = pressed,
                        winit::event::MouseButton::Middle => ws.context.mouse_middle = pressed,
                        _ => {}
                    }
                }
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ws) = self.state.as_mut() {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            ws.context.mouse_wheel_dx += x;
                            ws.context.mouse_wheel_dy += y;
                        }
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            ws.context.mouse_wheel_dx += pos.x as f32;
                            ws.context.mouse_wheel_dy += pos.y as f32;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Main entry point for window-based rendering.
pub fn run_window<C, R>(
    mut initial_context_fn: impl FnMut(&mut AlanWindowContext) + 'static,
    context_fn: C,
    gpgpu_shader_fn: R,
) -> Result<(), AlanError>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32> + 'static,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU> + 'static,
{
    let mut context = AlanWindowContext {
        window: None,
        start: None,
        buffer_width: None,
        mouse_x: None,
        mouse_y: None,
        mouse_left: false,
        mouse_right: false,
        mouse_middle: false,
        mouse_wheel_dx: 0.0,
        mouse_wheel_dy: 0.0,
        cursor_visible: true,
        transparent: false,
    };
    initial_context_fn(&mut context);
    let config = winit::window::Window::default_attributes().with_transparent(context.transparent);
    let event_loop = winit::event_loop::EventLoop::<()>::new()
        .map_err(|e| AlanError {
            message: format!("Failed to create event loop: {}", e),
        })?;
    let mut manager = WindowManager {
        window: None,
        state: None,
        init: Some(WindowInit {
            config,
            context,
            context_fn: Box::new(context_fn),
            gpgpu_shader_fn: Box::new(gpgpu_shader_fn),
        }),
    };
    event_loop.run_app(&mut manager).map_err(|e| AlanError {
        message: format!("Event loop error: {}", e),
    })?;
    Ok(())
}

/// Accessor functions for AlanWindowContext, used by JS bindings
pub fn context_width(ctx: &AlanWindowContext) -> u32 {
    ctx.width()
}

pub fn context_height(ctx: &AlanWindowContext) -> u32 {
    ctx.height()
}

pub fn context_buffer_width(ctx: &AlanWindowContext) -> u32 {
    ctx.buffer_width()
}

pub fn context_runtime(ctx: &AlanWindowContext) -> u32 {
    ctx.runtime()
}

pub fn context_mouse_x(ctx: &mut AlanWindowContext) -> u32 {
    ctx.mouse_x()
}

pub fn context_mouse_y(ctx: &mut AlanWindowContext) -> u32 {
    ctx.mouse_y()
}

pub fn context_mouse_left(ctx: &mut AlanWindowContext) -> u32 {
    ctx.mouse_left()
}

pub fn context_mouse_right(ctx: &mut AlanWindowContext) -> u32 {
    ctx.mouse_right()
}

pub fn context_mouse_middle(ctx: &mut AlanWindowContext) -> u32 {
    ctx.mouse_middle()
}

pub fn context_mouse_wheel_x(ctx: &mut AlanWindowContext) -> f32 {
    ctx.mouse_wheel_x()
}

pub fn context_mouse_wheel_y(ctx: &mut AlanWindowContext) -> f32 {
    ctx.mouse_wheel_y()
}

pub fn context_cursor_visible(ctx: &mut AlanWindowContext) {
    ctx.cursor_visible();
}

pub fn context_cursor_invisible(ctx: &mut AlanWindowContext) {
    ctx.cursor_invisible();
}

pub fn context_transparent(ctx: &mut AlanWindowContext) {
    ctx.transparent();
}

pub fn context_opaque(ctx: &mut AlanWindowContext) {
    ctx.opaque();
}

/// Accessor functions for AlanWindowFrame, used by JS bindings
pub fn frame_context(f: &AlanWindowFrame) -> GBuffer {
    f.context.clone()
}

pub fn frame_framebuffer(f: &AlanWindowFrame) -> GBuffer {
    f.framebuffer.clone()
}

pub fn frame_width(f: &AlanWindowFrame) -> u32 {
    f.width
}

pub fn frame_height(f: &AlanWindowFrame) -> u32 {
    f.height
}
