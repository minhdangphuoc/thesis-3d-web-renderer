use std::collections::HashMap;
use std::iter;
use wgpu::util::DeviceExt;
use wgpu::BindGroupLayout;
use winit::{event::*, window::Window};

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::model::{DrawModel, Model};
use crate::pipeline::Pipeline;
use crate::texture::{self, Texture};
use crate::utils::{Instance, InstanceRaw, Vertex};
use cgmath::prelude::*;

use crate::resources;

pub struct State {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    window: Window,
    
    // Camera
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_controller: CameraController,

    // Depth Buffer
    depth_texture: Texture,

    // instance
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    // Model
    model: Model,
}

impl State {
    pub async fn new(window: Window, url: &str) -> Self {
        let size = window.inner_size();

        // Backends: Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
            flags: Default::default(),
            gles_minor_version: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let mut surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format.remove_srgb_suffix(),
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()],
        };

        surface.configure(&device, &config);

        // Instance
        const SPACE_BETWEEN: f32 = 3.0;
        const NUM_INSTANCES_PER_ROW: u32 = 1;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Camera
        let camera = Camera::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
            5.0,
            2.0,
            10.0,
        );

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        // End: Camera

        // Depth texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let clear_color = wgpu::Color::BLACK;

        // Model Loader

        let model = resources::load_model(url, &device, &queue)
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            surface,
            device,
            queue,
            config,
            clear_color,
            size,
            window,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            camera_controller,
            depth_texture,
            instances,
            instance_buffer,
            model,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller
            .process_events(event, &mut self.camera);
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => true,
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // self.camera_controller.update_camera();
        self.camera_uniform.update_view_proj(&self.camera);
        self.camera.update();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.size
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor{
                format: Some(self.config.format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pipelines = HashMap::new();
        self.model.meshes.iter().for_each(|mesh| {
            let render_pipeline = Pipeline::new(
                &(mesh.name),
                &self.device,
                "Shader",
                include_str!("shaders/texture.wgsl"),
                &wgpu::PolygonMode::Fill,
                &self.config,
                &[
                    &self.model.materials[mesh.material].texture_bind_group_layout,
                    &self.camera_bind_group_layout,
                ],
                &[Vertex::desc(), InstanceRaw::desc()],
            );
            render_pipelines.insert(mesh.index, render_pipeline);
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: Default::default(),
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: Default::default(),
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: Default::default(),
                occlusion_query_set: Default::default(),
            });
            self.model.meshes.iter().for_each(|mesh| {
                match render_pipelines.get(&mesh.index) {
                    Some(render_pipeline) => {
                        render_pass.set_pipeline(&render_pipeline.get_pipeline());
                        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                        let material = &self.model.materials[mesh.material];
                        render_pass.draw_mesh_instanced(
                            &mesh,
                            &material,
                            0..1 as u32,
                            &self.camera_bind_group,
                        );
                    }
                    None => {}
                }
            });
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
