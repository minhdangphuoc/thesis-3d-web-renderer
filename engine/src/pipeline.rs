use wgpu::{util::DeviceExt, BindGroupLayout, Device, ShaderModule};

use crate::{
    texture::{self, Texture},
    utils::{InstanceRaw, Vertex},
};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        label: &str,
        device: &Device,
        shader_label: &str,
        shader_source: &str,
        polygon_mode: &wgpu::PolygonMode,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_layouts: &[wgpu::VertexBufferLayout],
    ) -> Self {
        let module = Self::create_shader(device, shader_label, shader_source);
        let layout = Self::create_new_layout(
            device,
            &format!("{label} Pipeline Layout"),
            bind_group_layouts,
        );
        let pipeline = Self::create_new_pipeline(
            device,
            &format!("{label} Pipeline"),
            &module,
            &polygon_mode,
            config,
            &layout,
            &vertex_layouts,
        );
        return Self { pipeline: pipeline };
    }

    pub fn create_shader(device: &Device, label: &str, url: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(url.into()),
        })
    }

    pub fn create_new_layout(
        device: &Device,
        label: &str,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        return device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: bind_group_layouts,
            push_constant_ranges: &[],
        });
    }

    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        return &self.pipeline;
    }

    fn create_new_pipeline(
        device: &Device,
        label: &str,
        module: &wgpu::ShaderModule,
        polygon_mode: &wgpu::PolygonMode,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        vertex_layouts: &[wgpu::VertexBufferLayout],
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&label),
            layout: Some(&layout),

            // Change to dynamic code
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: "vs_main",   // 1.
                buffers: &vertex_layouts, // 2.
            },

            // Change to dynamic code
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),

                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: *polygon_mode, // Poligon mode
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None, // 5.
        })
    }
}
