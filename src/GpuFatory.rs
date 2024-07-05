use std::borrow::Cow;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BlendState, Buffer, BufferBinding,
    BufferDescriptor, BufferUsages, FragmentState, FrontFace, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, VertexState,
};

use crate::GfxState;
pub struct GpuFactory {
    pub bind_group: Vec<BindGroup>,
    pub bind_group_layout: Vec<BindGroupLayout>,
    pub pipeline: Vec<RenderPipeline>,
    pub vertex_buffer: Vec<Buffer>,
    pub index_buffer: Vec<Buffer>,
    pub uniform_buffer: Vec<Buffer>,
    pub pipeline_layout: Vec<PipelineLayout>,
    pub shader: Vec<wgpu::ShaderModule>,
}

impl GpuFactory {
    pub fn new(app: &GfxState) {
        let code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/sky.wgsl"));
        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
            });
        let uniform_data = TheFirstUniformBuffer {
            width: app.surface_config.width,
            height: app.surface_config.height,
        };
        let uniform_buffer: Buffer = app.device.create_buffer(&BufferDescriptor {
            label: Some("first buffer"),
            size: std::mem::size_of::<TheFirstUniformBuffer>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        {
            uniform_buffer
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::bytes_of(&uniform_data));
            uniform_buffer.unmap();
        }

        let bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let bind_group = app.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let pipeline_layout = app
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = app
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "display_vs",
                    buffers: &[],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    front_face: FrontFace::Ccw,
                    polygon_mode: PolygonMode::Fill,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "display_fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render frame"),
            });

        println!("Creating render pass");
        let frame = app.surface.get_current_texture().unwrap();
        let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("display pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            render_pass.draw(0..6, 0..1);
            println!("Drawing");
        };

        let command_buffer = encoder.finish();
        app.queue.submit(Some(command_buffer));
        frame.present();
    }
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct TheFirstUniformBuffer {
    width: u32,
    height: u32,
}
