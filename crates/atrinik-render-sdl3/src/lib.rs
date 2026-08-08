// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

//! Thin SDL3/wgpu presentation bridge. It owns no scene, resources, event loop,
//! or renderer policy and must be called on the SDL video thread.

use atrinik_render_api::TargetKind;
use sdl3::video::Window;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationProof {
    pub width: u32,
    pub height: u32,
    pub adapter_name: String,
    pub backend: String,
    pub format: String,
}

/// Creates a resizable SDL window and presents one hardware frame through the
/// same bridge used by an embedded caller-provided SDL window.
pub fn present_window(width: u32, height: u32) -> Result<PresentationProof, Error> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidTarget);
    }
    let sdl = sdl3::init().map_err(|error| Error::Sdl(error.to_string()))?;
    let video = sdl.video().map_err(|error| Error::Sdl(error.to_string()))?;
    let window = video
        .window("Atrinik renderer proof", width, height)
        .position_centered()
        .resizable()
        .build()
        .map_err(|error| Error::Sdl(error.to_string()))?;
    present_existing(&window, TargetKind::Window)
}

/// Presents one frame into a caller-owned SDL window. The borrow prevents the
/// native surface from outliving its window.
pub fn present_existing(window: &Window, kind: TargetKind) -> Result<PresentationProof, Error> {
    if !matches!(kind, TargetKind::Window | TargetKind::Embedded) {
        return Err(Error::InvalidTarget);
    }
    let (width, height) = window.size_in_pixels();
    if width == 0 || height == 0 {
        return Err(Error::InvalidTarget);
    }
    let instance =
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
    let surface = raw_handle_bridge::create_surface(&instance, window).map_err(Error::Wgpu)?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
        apply_limit_buckets: false,
    }))
    .map_err(|error| Error::Wgpu(error.to_string()))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Atrinik SDL presentation device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::Performance,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| Error::Wgpu(error.to_string()))?;
    let capabilities = surface.get_capabilities(&adapter);
    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .or_else(|| capabilities.formats.first().copied())
        .ok_or_else(|| Error::Wgpu("surface exposes no formats".to_owned()))?;
    let present_mode = if capabilities
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
    {
        wgpu::PresentMode::Fifo
    } else {
        *capabilities
            .present_modes
            .first()
            .ok_or_else(|| Error::Wgpu("surface exposes no presentation modes".to_owned()))?
    };
    let alpha_mode = *capabilities
        .alpha_modes
        .first()
        .ok_or_else(|| Error::Wgpu("surface exposes no alpha modes".to_owned()))?;
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode,
            alpha_mode,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        },
    );
    let frame = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
            return Err(Error::TransientSurface);
        }
        wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
            surface.configure(
                &device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    color_space: wgpu::SurfaceColorSpace::Auto,
                    width,
                    height,
                    present_mode,
                    alpha_mode,
                    view_formats: Vec::new(),
                    desired_maximum_frame_latency: 2,
                },
            );
            return Err(Error::ReconfiguredSurface);
        }
        other => {
            return Err(Error::Wgpu(format!(
                "surface acquisition failed: {other:?}"
            )));
        }
    };
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Atrinik SDL proof encoder"),
    });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Atrinik SDL proof pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.03,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit(Some(encoder.finish()));
    queue.present(frame);
    let info = adapter.get_info();
    Ok(PresentationProof {
        width,
        height,
        adapter_name: info.name,
        backend: format!("{:?}", info.backend),
        format: format!("{format:?}"),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidTarget,
    TransientSurface,
    ReconfiguredSurface,
    Sdl(String),
    Wgpu(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SDL renderer bridge error: {self:?}")
    }
}

impl std::error::Error for Error {}

mod raw_handle_bridge {
    use sdl3::video::Window;
    use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

    /// SDL restricts Window operations to its video thread. wgpu's surface API
    /// requires a Send + Sync handle source even though it copies the handles
    /// during this synchronous call. This wrapper never escapes the call, and
    /// the returned surface retains the original Window borrow.
    struct BorrowedHandle<'a>(&'a Window);

    // SAFETY: the wrapper is constructed and consumed on the SDL video thread,
    // never shared or stored, and only delegates immutable raw-handle access.
    unsafe impl Send for BorrowedHandle<'_> {}
    // SAFETY: identical constraint; no SDL operation crosses a thread boundary.
    unsafe impl Sync for BorrowedHandle<'_> {}

    impl HasWindowHandle for BorrowedHandle<'_> {
        fn window_handle(&self) -> Result<wgpu::rwh::WindowHandle<'_>, wgpu::rwh::HandleError> {
            self.0.window_handle()
        }
    }

    impl HasDisplayHandle for BorrowedHandle<'_> {
        fn display_handle(&self) -> Result<wgpu::rwh::DisplayHandle<'_>, wgpu::rwh::HandleError> {
            self.0.display_handle()
        }
    }

    pub fn create_surface<'a>(
        instance: &wgpu::Instance,
        window: &'a Window,
    ) -> Result<wgpu::Surface<'a>, String> {
        instance
            .create_surface(BorrowedHandle(window))
            .map_err(|error| error.to_string())
    }
}
