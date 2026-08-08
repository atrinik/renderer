// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use atrinik_render_api::{
    Error as RenderError, FrameOutput, RecoveryMetrics, RenderRequest, Renderer, TargetDescriptor,
    TargetKind, TargetLimits,
};
use atrinik_render_resources::{
    Error as ResourceError, ResourceBlob, ResourceFuture, ResourceProvider, ResourceRequest,
};
use atrinik_scene::{
    Digest256, Layer, ResourceId, Sampling, SceneBundle, SceneLimits, SemanticId, Sprite, Viewport,
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::RwLock};

#[derive(Default)]
pub struct FakeResourceProvider {
    blobs: RwLock<BTreeMap<(ResourceId, u64), Vec<u8>>>,
}

impl FakeResourceProvider {
    /// Inserts a versioned immutable blob.
    ///
    /// # Errors
    /// Returns [`ResourceError::Unavailable`] if another test poisoned the lock.
    pub fn insert(
        &self,
        id: ResourceId,
        revision: u64,
        bytes: Vec<u8>,
    ) -> Result<(), ResourceError> {
        self.blobs
            .write()
            .map_err(|_| ResourceError::Unavailable)?
            .insert((id, revision), bytes);
        Ok(())
    }
}

impl ResourceProvider for FakeResourceProvider {
    fn load(&self, request: ResourceRequest) -> ResourceFuture<'_> {
        Box::pin(async move {
            request.validate()?;
            let bytes = self
                .blobs
                .read()
                .map_err(|_| ResourceError::Unavailable)?
                .get(&(request.id, request.revision))
                .cloned()
                .ok_or(ResourceError::NotFound)?;
            if Sha256::digest(&bytes).as_slice() != request.digest.0 {
                return Err(ResourceError::DigestMismatch);
            }
            ResourceBlob::new(bytes, request.maximum_bytes)
        })
    }
}

/// Deterministic software reference used by contract tests. It is intentionally
/// simple: sprites are solid, half-open rectangles in logical pixel space.
pub struct ReferenceRenderer {
    target: TargetDescriptor,
}

impl ReferenceRenderer {
    /// Creates a bounded renderer-neutral reference target.
    ///
    /// # Errors
    /// Returns a target error when the descriptor is invalid.
    pub fn new(target: TargetDescriptor) -> Result<Self, RenderError> {
        Ok(Self {
            target: target.validate(TargetLimits::default())?,
        })
    }
}

impl Renderer for ReferenceRenderer {
    fn target(&self) -> TargetDescriptor {
        self.target
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.target = TargetDescriptor {
            width,
            height,
            ..self.target
        }
        .validate(TargetLimits::default())?;
        Ok(())
    }

    fn render(&mut self, request: RenderRequest<'_>) -> Result<FrameOutput, RenderError> {
        if request.scene.viewport().width != self.target.width
            || request.scene.viewport().height != self.target.height
        {
            return Err(RenderError::InvalidTarget);
        }
        let pixel_count = pixel_count(self.target.width, self.target.height)?;
        let clear = color_to_rgba8(request.scene.clear_color());
        let mut output = FrameOutput {
            width: self.target.width,
            height: self.target.height,
            rgba8: clear.repeat(pixel_count),
            semantic_ids: vec![0; pixel_count],
            depth: vec![i16::MIN; pixel_count],
            coverage: vec![0; pixel_count],
            metrics: RecoveryMetrics::default(),
        };
        for sprite in request.scene.sprites() {
            raster_sprite(&mut output, sprite);
        }
        output.validate()?;
        Ok(output)
    }
}

fn pixel_count(width: u32, height: u32) -> Result<usize, RenderError> {
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(RenderError::LimitExceeded)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn raster_sprite(output: &mut FrameOutput, sprite: &Sprite) {
    let left = sprite.x.max(0.0).floor() as u32;
    let top = sprite.y.max(0.0).floor() as u32;
    let right = (sprite.x + sprite.width).max(0.0).ceil() as u32;
    let bottom = (sprite.y + sprite.height).max(0.0).ceil() as u32;
    let source = color_to_rgba8(sprite.color);
    for y in top.min(output.height)..bottom.min(output.height) {
        for x in left.min(output.width)..right.min(output.width) {
            let index = y as usize * output.width as usize + x as usize;
            blend(&mut output.rgba8[index * 4..index * 4 + 4], source);
            output.semantic_ids[index] = sprite.semantic_id.0;
            output.depth[index] = sprite.depth;
            output.coverage[index] = source[3];
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn color_to_rgba8(color: [f32; 4]) -> [u8; 4] {
    color.map(|channel| (channel.mul_add(255.0, 0.5)).floor() as u8)
}

fn blend(destination: &mut [u8], source: [u8; 4]) {
    let alpha = u32::from(source[3]);
    let inverse = 255 - alpha;
    for channel in 0..3 {
        destination[channel] = u8::try_from(
            (u32::from(source[channel]) * alpha + u32::from(destination[channel]) * inverse + 127)
                / 255,
        )
        .expect("8-bit blend channel remains in range");
    }
    destination[3] = u8::try_from(alpha + (u32::from(destination[3]) * inverse + 127) / 255)
        .expect("8-bit alpha remains in range");
}

#[must_use]
pub fn synthetic_digest() -> Digest256 {
    Digest256(Sha256::digest(b"atrinik-renderer-synthetic-resource-v1").into())
}

/// Builds the repository-owned visual contract scene at an explicit size.
///
/// # Errors
/// Returns a scene validation error for dimensions outside scene bounds.
#[must_use = "synthetic scene construction can fail"]
#[allow(clippy::cast_precision_loss)]
pub fn synthetic_scene(width: u32, height: u32) -> Result<SceneBundle, atrinik_scene::Error> {
    SceneBundle::new(
        1,
        1_000,
        Viewport {
            width,
            height,
            scale_milli: 1_000,
        },
        [0.02, 0.03, 0.05, 1.0],
        [
            Sprite {
                semantic_id: SemanticId(1),
                resource_id: ResourceId(1),
                resource_revision: 1,
                resource_digest: synthetic_digest(),
                x: 1.0,
                y: 1.0,
                width: (width / 2).max(1) as f32,
                height: (height / 2).max(1) as f32,
                depth: 0,
                painter_order: 0,
                color: [0.85, 0.12, 0.18, 1.0],
                sampling: Sampling::Nearest,
                layer: Layer::World,
            },
            Sprite {
                semantic_id: SemanticId(2),
                resource_id: ResourceId(1),
                resource_revision: 1,
                resource_digest: synthetic_digest(),
                x: (width / 3) as f32,
                y: (height / 3) as f32,
                width: (width / 2).max(1) as f32,
                height: (height / 2).max(1) as f32,
                depth: 1,
                painter_order: 1,
                color: [0.1, 0.55, 0.95, 0.75],
                sampling: Sampling::Linear,
                layer: Layer::Effect,
            },
        ],
        SceneLimits::default(),
    )
}

#[must_use]
pub const fn offscreen_target(width: u32, height: u32) -> TargetDescriptor {
    TargetDescriptor {
        kind: TargetKind::Offscreen,
        width,
        height,
        backend: atrinik_render_api::BackendPreference::Automatic,
    }
}

#[cfg(test)]
mod tests {
    use super::{FakeResourceProvider, ReferenceRenderer, offscreen_target, synthetic_scene};
    use atrinik_render_api::{RenderRequest, Renderer};
    use std::sync::Arc;

    #[test]
    fn reference_output_is_exact_and_repeatable() {
        let scene = synthetic_scene(16, 16).unwrap();
        let request = RenderRequest {
            scene: &scene,
            resources: Arc::new(FakeResourceProvider::default()),
        };
        let first = ReferenceRenderer::new(offscreen_target(16, 16))
            .unwrap()
            .render(request.clone())
            .unwrap();
        let second = ReferenceRenderer::new(offscreen_target(16, 16))
            .unwrap()
            .render(request)
            .unwrap();
        assert_eq!(first, second);
        assert!(first.semantic_ids.contains(&1));
        assert!(first.semantic_ids.contains(&2));
    }
}
