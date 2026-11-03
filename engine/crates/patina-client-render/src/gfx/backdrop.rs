//! The title screen's background, under the menus: the launcher's Mono
//! banner (`client_settings::backdrop`), covering the window.
use crate::client_settings::BackdropImage;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    size: [f32; 2],
    image: [f32; 2],
    time: f32,
    picture: f32,
    anchor: f32,
    pad: f32,
}

pub(super) struct Backdrop {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    nearest: wgpu::Sampler,
    linear: wgpu::Sampler,
    group: wgpu::BindGroup,
    /// The shown `client_settings::backdrop` generation and picture.
    generation: u64,
    image: Option<std::sync::Arc<BackdropImage>>,
}

fn sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Title background sampling"),
        mag_filter: filter,
        min_filter: filter,
        ..Default::default()
    })
}
fn bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Title background resources"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> wgpu::TextureView {
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Title background picture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            pixels,
        )
        .create_view(&Default::default())
}

impl Backdrop {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Title background parameters"),
            size: size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Title background bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Title background shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("backdrop.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Title background layout"),
            bind_group_layouts: &[Some(&layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Title background"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_backdrop"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_backdrop"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let nearest = sampler(device, wgpu::FilterMode::Nearest);
        let linear = sampler(device, wgpu::FilterMode::Linear);
        let view = upload(device, queue, 1, 1, &[0, 0, 0, 255]);
        let group = bind(device, &layout, &uniform, &view, &linear);
        Self {
            pipeline,
            layout,
            uniform,
            nearest,
            linear,
            group,
            generation: 0,
            image: None,
        }
    }

    /// Upload the chosen picture when it changed, and this frame's parameters.
    /// `time` is the background's running time in seconds.
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        time: f32,
    ) {
        let (generation, image) = crate::client_settings::backdrop();
        if generation != self.generation {
            self.generation = generation;
            if let Some(image) = &image {
                let view = upload(device, queue, image.width, image.height, &image.pixels);
                let filter = if image.blocky {
                    &self.nearest
                } else {
                    &self.linear
                };
                self.group = bind(device, &self.layout, &self.uniform, &view, filter);
            }
            self.image = image;
        }
        let params = Params {
            size: [width as f32, height.max(1) as f32],
            image: self
                .image
                .as_ref()
                .map_or([1.0; 2], |i| [i.width as f32, i.height as f32]),
            time,
            picture: f32::from(u8::from(self.image.is_some())),
            anchor: f32::from(u8::from(self.image.as_ref().is_some_and(|i| i.blocky))),
            pad: 0.0,
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&params));
    }

    pub(super) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// How long the background has run, in seconds at normal speed: the Panorama
/// Scroll Speed option scales it, and changing the speed never jumps.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    last: f32,
    time: f32,
}
impl Clock {
    pub fn update(&mut self, now: f32, speed: f32) -> f32 {
        self.time += (now - self.last).max(0.0) * speed;
        self.last = now;
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_scales_with_the_speed_and_never_jumps() {
        let mut clock = Clock::default();
        assert_eq!(clock.update(1.0, 1.0), 1.0);
        assert_eq!(clock.update(3.0, 0.5), 2.0);
        assert_eq!(clock.update(3.0, 4.0), 2.0);
    }
    #[test]
    fn the_shader_parses() {
        naga::front::wgsl::parse_str(include_str!("backdrop.wgsl")).unwrap();
    }
}
