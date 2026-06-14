use std::{ops::Deref, time::Duration};

use cgmath::One;
use compio::runtime::spawn;
use wgpu::util::DeviceExt;
use winio::prelude::*;

use crate::{Error, Result};

struct SurfaceData {
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl SurfaceData {
    fn new(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface: wgpu::Surface<'static>,
    ) -> Self {
        let surface_format = surface.get_capabilities(adapter).formats[0];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                model_view_proj: cgmath::Matrix4::one().into(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cube Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("wgpu/shader.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cube Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cube Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            surface,
            surface_format,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            render_pipeline,
        }
    }

    fn rotate(&self, queue: &wgpu::Queue, rotation_angle: f32, aspect_ratio: f32) {
        let model = cgmath::Matrix4::from_angle_y(cgmath::Rad(rotation_angle));
        let view = cgmath::Matrix4::look_to_rh(
            cgmath::Point3::new(1.5, 1.5, 1.5),
            cgmath::Vector3::new(-1.0, -1.0, -1.0),
            cgmath::Vector3::unit_y(),
        );
        let projection = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 100.0);
        let mvp = projection * view * model;

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                model_view_proj: mvp.into(),
            }]),
        );
    }

    fn configure(&self, device: &wgpu::Device, size: Size) -> Result<()> {
        let width = (size.width as u32).max(1);
        let height = (size.height as u32).max(1);
        let needs_update = self
            .surface
            .get_configuration()
            .map(|c| c.width != width || c.height != height)
            .unwrap_or(true);
        if needs_update {
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                view_formats: vec![self.surface_format.add_srgb_suffix()],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width,
                height,
                desired_maximum_frame_latency: 2,
                present_mode: wgpu::PresentMode::AutoVsync,
            };
            self.surface.configure(device, &surface_config);
        }
        Ok(())
    }
}

pub struct WgpuPage {
    window: Child<TabViewItem>,
    canvas: Child<WgpuCanvas>,
    angle: f32,
    instance: wgpu::Instance,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    queue: wgpu::Queue,
    surface: Option<SurfaceData>,
}

impl WgpuPage {
    fn configure(&mut self, size: Size) -> Result<()> {
        if self.surface.is_none() {
            self.surface = self
                .canvas
                .create_surface(&self.instance)
                .ok()
                .map(|s| SurfaceData::new(&self.device, &self.adapter, s));
        }
        if let Some(surface) = &self.surface {
            surface.configure(&self.device, size)?;
        }
        Ok(())
    }

    fn rotate(&mut self, angle: f32, aspect_ratio: f32) {
        if let Some(surface) = &self.surface {
            surface.rotate(&self.queue, angle, aspect_ratio);
        }
    }
}

#[derive(Debug)]
pub enum WgpuPageEvent {}

#[derive(Debug)]
pub enum WgpuPageMessage {
    Tick,
}

impl Component for WgpuPage {
    type Error = Error;
    type Event = WgpuPageEvent;
    type Init<'a> = ();
    type Message = WgpuPageMessage;

    async fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: "WGPU",
            },
            canvas: WgpuCanvas = (&window),
        }

        spawn({
            let sender = sender.clone();
            async move {
                let mut interval = compio::time::interval(Duration::from_millis(10));
                loop {
                    interval.tick().await;
                    sender.post(WgpuPageMessage::Tick);
                }
            }
        })
        .detach();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        let surface = canvas
            .create_surface(&instance)
            .ok()
            .map(|s| SurfaceData::new(&device, &adapter, s));
        Ok(Self {
            window,
            canvas,
            angle: 0.0,
            instance,
            device,
            adapter,
            queue,
            surface,
        })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WgpuPageMessage::Tick => {
                self.angle += 0.01;
                self.angle %= std::f32::consts::TAU;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let prev_size = self.canvas.size()?;
        let size = self.window.size()?;
        if prev_size != size || self.surface.is_none() {
            self.canvas.set_rect(size.into())?;
            self.configure(size)?;
        }

        if size == Size::zero() {
            return Ok(());
        }

        self.rotate(self.angle, size.width as f32 / size.height as f32);

        let Some(data) = &self.surface else {
            return Ok(());
        };
        let surface_texture = match data.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                self.configure(size)?;
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.take();
                self.configure(size)?;
                return Ok(());
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(data.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&data.render_pipeline);
            render_pass.set_bind_group(0, &data.bind_group, &[]);
            render_pass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
            render_pass.set_index_buffer(data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        surface_texture.present();

        Ok(())
    }
}

impl Deref for WgpuPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0, // location
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // color
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    // front
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    }, // 0
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    }, // 1
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    }, // 2
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    }, // 3
    // back
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    }, // 4
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    }, // 5
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 1.0, 1.0],
    }, // 6
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.5, 0.5],
    }, // 7
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // front
    4, 6, 5, 4, 7, 6, // back
    0, 7, 4, 0, 3, 7, // left
    1, 5, 6, 1, 6, 2, // right
    3, 2, 6, 3, 6, 7, // top
    0, 4, 5, 0, 5, 1, // bottom
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model_view_proj: [[f32; 4]; 4],
}
