use wgpu::util::DeviceExt;

const GRID_SIZE: u32 = 64;  // Start small for debugging
const WORKGROUP_SIZE: u32 = 8;

pub struct ConwayCompute {
    compute_pipeline: wgpu::ComputePipeline,
    state_buffers: [wgpu::Buffer; 2],  // Ping-pong buffers
    current_buffer: usize,
    bind_groups: [wgpu::BindGroup; 2],
}

impl ConwayCompute {
    pub fn new(device: &wgpu::Device) -> Self {
        // Create compute shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Conway Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("conway.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Conway Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
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

        // Create initial random state
        let initial_state: Vec<u32> = (0..GRID_SIZE * GRID_SIZE)
            .map(|_| if rand::random::<f32>() > 0.7 { 1 } else { 0 })
            .collect();

        // Create ping-pong buffers
        let state_buffers = [
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Conway State Buffer A"),
                contents: bytemuck::cast_slice(&initial_state),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Conway State Buffer B"),
                size: (initial_state.len() * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        // Create bind groups
        let bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Conway Bind Group A"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: state_buffers[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: state_buffers[1].as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Conway Bind Group B"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: state_buffers[1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: state_buffers[0].as_entire_binding(),
                    },
                ],
            }),
        ];

        Self {
            compute_pipeline,
            state_buffers,
            current_buffer: 0,
            bind_groups,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Conway Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_groups[self.current_buffer], &[]);
        
        let workgroups_x = (GRID_SIZE + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
        let workgroups_y = (GRID_SIZE + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
        
        compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        
        drop(compute_pass);
        
        // Swap buffers for next iteration
        self.current_buffer = 1 - self.current_buffer;
    }
    
    pub fn get_current_buffer(&self) -> &wgpu::Buffer {
        &self.state_buffers[1 - self.current_buffer]  // The one we just wrote to
    }
}