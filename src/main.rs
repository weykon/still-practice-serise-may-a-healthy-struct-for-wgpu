use std::sync::Arc;
mod GpuFatory;
use anyhow::{anyhow, Context};
use camera::Camera;
use wgpu::{Adapter, Color, LoadOp, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, WindowEvent},
    event_loop::{self, ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};
use GpuFatory::GpuFactory;
mod camera;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app_entry = EntryOn::Loading;
    let _ = event_loop.run_app(&mut app_entry);
}

struct GfxState {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'static>,
    pub queue: wgpu::Queue,
    pub camera: camera::Camera,
    pub surface_config: wgpu::SurfaceConfiguration,
}

enum EntryOn {
    Loading,
    Ready(GfxState),
}

impl GfxState {
    async fn new(window: Arc<Window>) -> Self {
        let size: winit::dpi::PhysicalSize<u32> = window.inner_size();
        let wgpu_instance = wgpu::Instance::default();
        let surface = wgpu_instance.create_surface(window.clone()).unwrap();
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::util::power_preference_from_env()
                    .unwrap_or(wgpu::PowerPreference::HighPerformance),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapters found on the system!");
        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        let base_dir = std::env::var("CARGO_MANIFEST_DIR");
        let _trace_path = if let Ok(base_dir) = base_dir {
            Some(std::path::PathBuf::from(&base_dir).join("WGPU_TRACE_ERROR"))
        } else {
            None
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|_| anyhow!("Failed to create device"))
            .unwrap();
        println!("Device created : {:?}", device.global_id());

        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        surface.configure(&device, &surface_config);
        println!("Gfx State Ready");

        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        Self {
            window,
            device,
            surface,
            camera,
            queue,
            surface_config,
        }
    }
}

impl ApplicationHandler for EntryOn {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Loading = self {
            let window = event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_active(false)
                        .with_inner_size(PhysicalSize::new(128, 128)),
                )
                .unwrap();
            let window = Arc::new(window);
            pollster::block_on(async move {
                println!("async block");
                let gfx_state = GfxState::new(window.clone()).await;
                *self = EntryOn::Ready(gfx_state);
                println!("Ready now!");
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Self::Ready(app) = self {
            match event {
                WindowEvent::Resized(size) => {
                    println!("Resized");
                    app.surface_config.width = size.width;
                    app.surface_config.height = size.height;
                    app.surface.configure(&app.device, &app.surface_config);
                    app.window.request_redraw();
                }
                WindowEvent::RedrawRequested { .. } => {
                    println!("RedrawRequested");
                    GpuFactory::new(app);
                }
                WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {
                    println!("KeyboardInput: {:?}", event.physical_key);
                }
                WindowEvent::CloseRequested => {
                    println!("CloseRequested");
                }
                _ => {}
            }
        } else {
            println!("Not ready yet! in Loading");
        }
    }
}
