// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use atrinik_render_api::{
    BackendPreference, Error, FrameOutput, RecoveryMetrics, RenderRequest, Renderer,
    TargetDescriptor, TargetKind, TargetLimits,
};
use atrinik_scene::{SceneBundle, Sprite};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, sync::mpsc};

const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterDescription {
    pub name: String,
    pub driver: String,
    pub backend: String,
    pub device_type: String,
}

pub struct WgpuRenderer {
    target: TargetDescriptor,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    adapter: AdapterDescription,
    metrics: RecoveryMetrics,
}

impl WgpuRenderer {
    /// Creates an offscreen Vulkan or Direct3D 12 renderer.
    ///
    /// # Errors
    /// Returns a typed target, adapter, or device creation error.
    pub fn new(target: TargetDescriptor) -> Result<Self, Error> {
        let target = target.validate(TargetLimits::default())?;
        if target.kind != TargetKind::Offscreen {
            return Err(Error::InvalidTarget);
        }
        let backends = match target.backend {
            BackendPreference::Automatic => wgpu::Backends::VULKAN | wgpu::Backends::DX12,
            BackendPreference::Vulkan => wgpu::Backends::VULKAN,
            BackendPreference::Direct3D12 => wgpu::Backends::DX12,
        };
        let mut instance_descriptor =
            wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        instance_descriptor.backends &= backends;
        let instance = wgpu::Instance::new(instance_descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }))
        .map_err(|_| Error::BackendUnavailable)?;
        let info = adapter.get_info();
        let adapter_description = AdapterDescription {
            name: info.name,
            driver: info.driver,
            backend: format!("{:?}", info.backend),
            device_type: format!("{:?}", info.device_type),
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Atrinik renderer device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(|error| Error::Internal(error.to_string()))?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Atrinik solid sprite shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Atrinik solid sprite pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Vertex::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OUTPUT_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            target,
            device,
            queue,
            pipeline,
            adapter: adapter_description,
            metrics: RecoveryMetrics::default(),
        })
    }

    #[must_use]
    pub fn adapter(&self) -> &AdapterDescription {
        &self.adapter
    }

    #[allow(clippy::too_many_lines)]
    fn render_rgba(&self, scene: &SceneBundle) -> Result<Vec<u8>, Error> {
        let size = wgpu::Extent3d {
            width: self.target.width,
            height: self.target.height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atrinik offscreen color target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OUTPUT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let vertices = vertices(scene);
        let vertex_buffer = create_buffer(
            &self.device,
            "Atrinik frame vertices",
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
        );
        let unpadded_row = self
            .target
            .width
            .checked_mul(4)
            .ok_or(Error::LimitExceeded)?;
        let padded_row = unpadded_row
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .ok_or(Error::LimitExceeded)?;
        let readback_size = u64::from(padded_row)
            .checked_mul(u64::from(self.target.height))
            .ok_or(Error::LimitExceeded)?;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Atrinik frame readback"),
            size: readback_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Atrinik frame encoder"),
            });
        {
            let clear = scene.clear_color();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Atrinik frame render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(clear[0]),
                            g: f64::from(clear[1]),
                            b: f64::from(clear[2]),
                            a: f64::from(clear[3]),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(
                0..u32::try_from(vertices.len()).map_err(|_| Error::LimitExceeded)?,
                0..1,
            );
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(self.target.height),
                },
            },
            size,
        );
        self.queue.submit(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ignored = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|error| Error::Internal(error.to_string()))?;
        receiver
            .recv()
            .map_err(|error| Error::Internal(error.to_string()))?
            .map_err(|error| Error::Internal(error.to_string()))?;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| Error::Internal(error.to_string()))?;
        let capacity = usize::try_from(unpadded_row)
            .ok()
            .and_then(|row| row.checked_mul(self.target.height as usize))
            .ok_or(Error::LimitExceeded)?;
        let mut rgba = Vec::with_capacity(capacity);
        for row in mapped.chunks_exact(padded_row as usize) {
            rgba.extend_from_slice(&row[..unpadded_row as usize]);
        }
        drop(mapped);
        readback.unmap();
        Ok(rgba)
    }
}

impl Renderer for WgpuRenderer {
    fn target(&self) -> TargetDescriptor {
        self.target
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.target = TargetDescriptor {
            width,
            height,
            ..self.target
        }
        .validate(TargetLimits::default())?;
        self.metrics.surface_reconfigurations += 1;
        Ok(())
    }

    fn render(&mut self, request: RenderRequest<'_>) -> Result<FrameOutput, Error> {
        if request.scene.viewport().width != self.target.width
            || request.scene.viewport().height != self.target.height
        {
            return Err(Error::InvalidTarget);
        }
        let rgba8 = self.render_rgba(request.scene)?;
        let (semantic_ids, depth, coverage) = semantic_planes(request.scene)?;
        let output = FrameOutput {
            width: self.target.width,
            height: self.target.height,
            rgba8,
            semantic_ids,
            depth,
            coverage,
            metrics: self.metrics,
        };
        output.validate()?;
        Ok(output)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn vertices(scene: &SceneBundle) -> Vec<Vertex> {
    let viewport = scene.viewport();
    let mut output = Vec::with_capacity(scene.sprites().len().saturating_mul(6));
    for sprite in scene.sprites() {
        let left = sprite.x.mul_add(2.0 / viewport.width as f32, -1.0);
        let right = (sprite.x + sprite.width).mul_add(2.0 / viewport.width as f32, -1.0);
        let top = 1.0 - sprite.y * 2.0 / viewport.height as f32;
        let bottom = 1.0 - (sprite.y + sprite.height) * 2.0 / viewport.height as f32;
        let vertex = |position| Vertex {
            position,
            color: sprite.color,
        };
        output.extend_from_slice(&[
            vertex([left, top]),
            vertex([left, bottom]),
            vertex([right, bottom]),
            vertex([left, top]),
            vertex([right, bottom]),
            vertex([right, top]),
        ]);
    }
    output
}

fn create_buffer(
    device: &wgpu::Device,
    label: &str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let size = u64::try_from(contents.len().max(4)).expect("buffer size fits u64");
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: true,
    });
    if !contents.is_empty() {
        buffer
            .slice(..contents.len() as u64)
            .get_mapped_range_mut()
            .expect("newly-created mapped buffer has a valid range")
            .copy_from_slice(contents);
    }
    buffer.unmap();
    buffer
}

type SemanticPlanes = (Vec<u32>, Vec<i16>, Vec<u8>);

fn semantic_planes(scene: &SceneBundle) -> Result<SemanticPlanes, Error> {
    let viewport = scene.viewport();
    let pixels = (viewport.width as usize)
        .checked_mul(viewport.height as usize)
        .ok_or(Error::LimitExceeded)?;
    let mut semantic = vec![0; pixels];
    let mut depth = vec![i16::MIN; pixels];
    let mut coverage = vec![0; pixels];
    for sprite in scene.sprites() {
        mark_sprite(
            viewport.width,
            viewport.height,
            sprite,
            &mut semantic,
            &mut depth,
            &mut coverage,
        );
    }
    Ok((semantic, depth, coverage))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn mark_sprite(
    width: u32,
    height: u32,
    sprite: &Sprite,
    semantic: &mut [u32],
    depth: &mut [i16],
    coverage: &mut [u8],
) {
    let left = sprite.x.max(0.0).floor() as u32;
    let top = sprite.y.max(0.0).floor() as u32;
    let right = (sprite.x + sprite.width).max(0.0).ceil() as u32;
    let bottom = (sprite.y + sprite.height).max(0.0).ceil() as u32;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = y as usize * width as usize + x as usize;
            semantic[index] = sprite.semantic_id.0;
            depth[index] = sprite.depth;
            coverage[index] = (sprite.color[3].mul_add(255.0, 0.5)).floor() as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WgpuRenderer;
    use atrinik_render_api::{RenderRequest, Renderer};
    use atrinik_render_testkit::{
        FakeResourceProvider, ReferenceRenderer, offscreen_target, synthetic_scene,
    };
    use std::sync::Arc;

    #[test]
    fn renders_offscreen_when_a_gpu_adapter_is_available() {
        let Ok(mut renderer) = WgpuRenderer::new(offscreen_target(16, 16)) else {
            eprintln!("skipping GPU smoke test: no supported adapter");
            return;
        };
        let scene = synthetic_scene(16, 16).unwrap();
        let request = RenderRequest {
            scene: &scene,
            resources: Arc::new(FakeResourceProvider::default()),
        };
        let frame = renderer.render(request.clone()).unwrap();
        let reference = ReferenceRenderer::new(offscreen_target(16, 16))
            .unwrap()
            .render(request)
            .unwrap();
        assert_eq!(frame.rgba8.len(), 16 * 16 * 4);
        assert!(frame.rgba8.windows(4).any(|pixel| pixel != [5, 8, 13, 255]));
        assert_eq!(frame.semantic_ids, reference.semantic_ids);
        assert_eq!(frame.depth, reference.depth);
        assert_eq!(frame.coverage, reference.coverage);
        let maximum_channel_difference = frame
            .rgba8
            .iter()
            .zip(reference.rgba8.iter())
            .map(|(actual, expected)| actual.abs_diff(*expected))
            .max()
            .unwrap();
        assert!(
            maximum_channel_difference <= 1,
            "maximum RGBA difference: {maximum_channel_difference}"
        );
    }

    #[test]
    fn shader_is_valid_portable_wgsl() {
        let source = include_str!("shader.wgsl");
        let module = naga::front::wgsl::parse_str(source).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap();
    }
}
