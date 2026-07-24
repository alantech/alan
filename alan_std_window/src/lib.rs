/// Alan module: @std/window
/// Provides window and GPU rendering primitives.
///
/// Import with:
///   type Window <-- '@std/window'
///   type Frame <-- '@std/window'
///   fn window <-- '@std/window'

/// Marker type for window-scope Rust bindings.
pub struct WindowBacking;

// Re-export GBuffer and GPGPU types from alan_std
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

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

/// Window context struct -- holds window state and input tracking.
pub struct AlanWindowContext {
    window: Option<Arc<Window>>,
    start: Option<std::time::Instant>,
    buffer_width: Option<u32>,
    mouse_x: Option<u32>,
    mouse_y: Option<u32>,
    mouse_left: bool,
    mouse_right: bool,
    mouse_middle: bool,
    mouse_wheel_dx: f32,
    mouse_wheel_dy: f32,
    cursor_visible: bool,
    transparent: bool,
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

/// Generic window handler -- closures are stored as type params, NOT boxed traits.
/// This avoids 'static and Send bounds entirely.
pub struct AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    config: WindowAttributes,
    context: AlanWindowContext,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    context_buffer: Option<GBuffer>,
    buffer: Option<GBuffer>,
    cached_surface_config: Option<wgpu::SurfaceConfiguration>,
    cached_size: PhysicalSize<u32>,
    context_fn: C,
    gpgpu_shader_fn: R,
    gpgpu_shaders: Option<Vec<GPGPU>>,
    inited: bool,
}

impl<C, R> AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    fn gpu_init(&mut self) {
        if self.context.start.is_none() {
            self.context.start = Some(std::time::Instant::now());
        }
        if self.surface.is_none() {
            self.surface = Some(
                alan_std::instance()
                    .create_surface(self.context.window.as_ref().unwrap().clone())
                    .unwrap(),
            );
        }
        if self.device.is_none() {
            let g = alan_std::gpu();
            self.device = Some(g.get_device().clone());
            self.queue = Some(g.get_queue().clone());
        }
        if self.context_buffer.is_none() {
            self.context_buffer = Some(create_empty_buffer(&storage_buffer_type(), &64, &4).unwrap());
        }
        if self.buffer.is_none() {
            let mut size = self.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            self.context.buffer_width = Some(if (4 * size.width).is_multiple_of(256) {
                4 * size.width
            } else {
                (4 * size.width) + (256 - ((4 * size.width) % 256))
            });
            // buffer_width is already in bytes (aligned). create_empty_buffer multiplies count * element_size,
            // so we pass byte_size / 4 as the count to get the correct total byte size.
            let buffer_byte_size = (self.context.buffer_width.unwrap() as u64) * (size.height as u64);
            self.buffer = Some(
                create_empty_buffer(&storage_buffer_type(), &((buffer_byte_size / 4) as i64), &4).unwrap(),
            );
        }
        if self.gpgpu_shaders.is_none() {
            let mut size = self.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            self.gpgpu_shaders = Some((self.gpgpu_shader_fn)(&AlanWindowFrame {
                context: self.context_buffer.as_ref().unwrap().clone(),
                framebuffer: self.buffer.as_ref().unwrap().clone(),
                width: size.width,
                height: size.height,
            }));
        }
        self.inited = true;
    }

    fn render_frame(&mut self) {
        if !self.inited {
            self.gpu_init();
        }
        // Clone the Arc early so we don't hold an immutable borrow on self.context
        let window = self.context.window.as_ref().unwrap().clone();
        window.set_cursor_visible(self.context.cursor_visible);
        window.set_transparent(self.context.transparent);
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let surface = self.surface.as_ref().unwrap();
        let g = alan_std::gpu();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        if self.cached_surface_config.is_none() || self.cached_size != size {
            let mut config = surface.get_default_config(&g.adapter, size.width, size.height).unwrap();
            config.usage = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            config.desired_maximum_frame_latency = 1;
            config.alpha_mode = if self.context.transparent {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };
            surface.configure(device, &config);
            self.cached_surface_config = Some(config);
            self.cached_size = size;
        }
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => return,
        };
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_array = (self.context_fn)(&mut self.context);
        let context_slice = &context_array[..];
        let context_ptr = context_slice.as_ptr();
        let context_u8_len = context_array.len() * 4;
        let context_u8: &[u8] =
            unsafe { std::slice::from_raw_parts(context_ptr as *const u8, context_u8_len) };
        let ctx_buf = self.context_buffer.as_ref().unwrap();
        queue.write_buffer(&**ctx_buf, 0, context_u8);
        let ggs = self.gpgpu_shaders.as_mut().unwrap();
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
        let framebuffer = self.buffer.as_ref().unwrap();
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &**framebuffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: self.context.buffer_width,
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

impl<C, R> ApplicationHandler for AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.context.window = Some(Arc::new(
            event_loop.create_window(self.config.clone()).unwrap(),
        ));
        self.context.window.as_ref().unwrap().request_redraw();
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.gpgpu_shaders = None;
                self.context.window = None;
                if let Some(b) = &self.buffer {
                    b.destroy();
                }
                self.buffer = None;
                if let Some(b) = &self.context_buffer {
                    b.destroy();
                }
                self.context_buffer = None;
                self.queue = None;
                self.device = None;
                self.surface = None;
                event_loop.exit();
            }
            WindowEvent::Resized(mut new_size) => {
                if event_loop.exiting() {
                    return;
                }
                if !self.inited {
                    return;
                }
                new_size.width = new_size.width.max(1);
                new_size.height = new_size.height.max(1);
                let buffer_width = if (4 * new_size.width) % 256 == 0 {
                    4 * new_size.width
                } else {
                    (4 * new_size.width) + (256 - ((4 * new_size.width) % 256))
                };
                // create_empty_buffer multiplies count * element_size, so pass byte_size / 4 as count
                let buffer_byte_size = (buffer_width as u64) * (new_size.height as u64);
                let new_buffer =
                    create_empty_buffer(&storage_buffer_type(), &((buffer_byte_size / 4) as i64), &4).unwrap();
                if let Some(b) = &self.buffer {
                    b.destroy();
                }
                self.buffer = Some(new_buffer);
                self.context.buffer_width = Some(buffer_width);
                self.gpgpu_shaders = Some((self.gpgpu_shader_fn)(&AlanWindowFrame {
                    context: self.context_buffer.as_ref().unwrap().clone(),
                    framebuffer: self.buffer.as_ref().unwrap().clone(),
                    width: new_size.width,
                    height: new_size.height,
                }));
                self.context.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if event_loop.exiting() {
                    return;
                }
                self.render_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.context.mouse_x.is_some() {
                    self.context.mouse_x = Some(position.x as u32);
                    self.context.mouse_y = Some(position.y as u32);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.context.mouse_left = pressed,
                    MouseButton::Right => self.context.mouse_right = pressed,
                    MouseButton::Middle => self.context.mouse_middle = pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.context.mouse_wheel_dx += x;
                        self.context.mouse_wheel_dy += y;
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.context.mouse_wheel_dx += pos.x as f32;
                        self.context.mouse_wheel_dy += pos.y as f32;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Main entry point for window-based rendering.
/// Closures are generic type params (NOT boxed traits), so NO 'static or Send bounds needed.
pub fn run_window<C, R>(
    mut initial_context_fn: impl FnMut(&mut AlanWindowContext),
    context_fn: C,
    gpgpu_shader_fn: R,
) -> Result<(), AlanError>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    let context = AlanWindowContext {
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

    let config = Window::default_attributes().with_transparent(context.transparent);

    let event_loop = EventLoop::new().map_err(|e| AlanError {
        message: format!("Failed to create event loop: {}", e),
    })?;

    let mut alan_window = AlanWindow {
        config,
        context,
        surface: None,
        device: None,
        queue: None,
        context_buffer: None,
        buffer: None,
        cached_surface_config: None,
        cached_size: PhysicalSize::new(0, 0),
        context_fn,
        gpgpu_shader_fn,
        gpgpu_shaders: None,
        inited: false,
    };

    // Run initial context function before entering event loop
    initial_context_fn(&mut alan_window.context);

    event_loop.run_app(&mut alan_window).map_err(|e| AlanError {
        message: format!("Event loop error: {}", e),
    })?;

    Ok(())
}

/// Accessor functions for AlanWindowContext
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

/// Accessor functions for AlanWindowFrame
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
