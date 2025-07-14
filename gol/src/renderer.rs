use crate::conway::ConwayCompute;
use crate::gpu::GpuContext;
use winit::window::Window;

pub struct Renderer {
    pub name: String,
    conway: ConwayCompute,
    display_pipeline: wgpu::RenderPipeline,
    display_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn new(name: &str, ctx: &GpuContext) -> Self {
        let conway = ConwayCompute::new(&ctx.device, &ctx.queue);

        // Create display shader
        let display_shader = &ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Display Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("display.wgsl").into()),
            });

        // Create bind group layout for Conway state buffer
        let display_bind_group_layout =
            &ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Display Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    }],
                });

        // Create display bind group
        let display_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Display Bind Group"),
            layout: &display_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(conway.get_current_texture_view()),
            }],
        });

        // Create display pipeline
        let display_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Display Pipeline Layout"),
                    bind_group_layouts: &[&display_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let display_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Display Pipeline"),
                layout: Some(&display_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &display_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[], // No vertex buffers - we generate fullscreen triangle in shader
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &display_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.surface.format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None, // No culling for fullscreen triangle
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            name: name.to_string(),
            conway,
            display_pipeline,
            display_bind_group,
        }
    }

    pub fn render(&mut self, window: &Window, ctx: &GpuContext) -> Result<(), wgpu::SurfaceError> {
        window.request_redraw();

        if !ctx.surface.is_configured() {
            return Ok(());
        }

        let output = ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("{:?} Renderer Encoder", self.name)),
            });

        // Step Conway's Game of Life
        self.conway.step(&mut encoder);

        // Render Conway's Game of Life
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Conway Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw fullscreen triangle with Conway's state
            render_pass.set_pipeline(&self.display_pipeline);
            render_pass.set_bind_group(0, &self.display_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // 3 vertices for fullscreen triangle
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
