use crate::gpu::GpuContext;
use winit::window::Window;

pub struct Renderer {
    pub name: String,
}

impl Renderer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn render(&mut self, window: &Window, ctx: &GpuContext) -> Result<(), wgpu::SurfaceError> {
        window.request_redraw();

        if !ctx.surface.is_configured() {
            return Ok(());
        }

        let output = ctx.surface.get_current_texture()?;

        let encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("{:?} Renderer Encoder", self.name)),
            });

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
