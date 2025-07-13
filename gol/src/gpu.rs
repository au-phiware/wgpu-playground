use std::cmp;
use std::{sync::Arc};

use anyhow::Result;
use wgpu::{Surface, Device, Queue, SurfaceConfiguration};
use winit::window::Window;

pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: SurfaceManager,
}

pub struct SurfaceManager {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    is_configured: bool,
}

impl SurfaceManager {
    pub fn is_configured(&self) -> bool {
        self.is_configured
    }

    fn configure(&mut self, device: &Device) {
        self.surface.configure(device, &self.config);
        self.is_configured = true;
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off, // Trace path
            })
            .await?;

        Ok(Self {
            device,
            queue,
            surface: SurfaceManager {
                window,
                surface,
                config,
                is_configured: false,
            },
        })
    }

    pub fn resize(&mut self) {
        let size = self.surface.window.inner_size();
        if size.width > 0 && size.height > 0 {
            self.surface.config.width = cmp::min(size.width, 2048);
            self.surface.config.height = cmp::min(size.height, 2048);
            self.surface.configure(&self.device);
        }
    }
}
