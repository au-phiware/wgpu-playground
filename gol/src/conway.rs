const GRID_SIZE: u32 = 64; // Start small for debugging
const WORKGROUP_SIZE: u32 = 16;

pub struct ConwayCompute {
    compute_pipeline: wgpu::ComputePipeline,
    state_views: [wgpu::TextureView; 2],
    current_texture: usize,
    bind_groups: [wgpu::BindGroup; 2],
}

impl ConwayCompute {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Create compute shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Conway Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("conway.wgsl").into()),
        });

        // Create bind group layout for textures
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Conway Texture Bind Group Layout"),
            entries: &[
                // Input texture (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                // Output storage texture (write)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Conway Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Conway Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Create initial state data as floats
        let initial_state: Vec<f32> = (0..GRID_SIZE * GRID_SIZE)
            .map(|_| if rand::random::<f32>() > 0.7 { 1.0 } else { 0.0 })
            .collect();

        // Create texture descriptor
        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Conway State Texture"),
            size: wgpu::Extent3d {
                width: GRID_SIZE,
                height: GRID_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        // Create ping-pong textures
        let state_textures = [
            device.create_texture(&texture_descriptor),
            device.create_texture(&texture_descriptor),
        ];

        // Initialize first texture with data
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &state_textures[0],
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&initial_state),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(GRID_SIZE * 4), // 4 bytes per f32
                rows_per_image: Some(GRID_SIZE),
            },
            wgpu::Extent3d {
                width: GRID_SIZE,
                height: GRID_SIZE,
                depth_or_array_layers: 1,
            },
        );

        let state_views = [
            state_textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
            state_textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ];

        // Create bind groups (ping-pong)
        let bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Conway Texture Bind Group A"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&state_views[0]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&state_views[1]),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Conway Texture Bind Group B"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&state_views[1]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&state_views[0]),
                    },
                ],
            }),
        ];

        Self {
            compute_pipeline,
            state_views,
            current_texture: 0,
            bind_groups,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Conway Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[self.current_texture], &[]);

        let workgroups_x = (GRID_SIZE + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
        let workgroups_y = (GRID_SIZE + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;

        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);

        drop(compute_pass);

        // Swap textures for next iteration
        self.current_texture = 1 - self.current_texture;
    }

    pub fn get_current_texture_view(&self) -> &wgpu::TextureView {
        &self.state_views[1 - self.current_texture] // The one we just wrote to
    }
}
