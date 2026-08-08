// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

use atrinik_render_api::{RenderRequest, Renderer};
use atrinik_render_testkit::{
    FakeResourceProvider, ReferenceRenderer, offscreen_target, synthetic_scene,
};
use sha2::{Digest, Sha256};
use std::{fmt::Write as _, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{{\"schema_version\":1,\"cases\":[");
    for (index, (name, width, height)) in [("overlap-square", 16, 16), ("unaligned-wide", 31, 17)]
        .into_iter()
        .enumerate()
    {
        let scene = synthetic_scene(width, height)?;
        let frame =
            ReferenceRenderer::new(offscreen_target(width, height))?.render(RenderRequest {
                scene: &scene,
                resources: Arc::new(FakeResourceProvider::default()),
            })?;
        let semantic = frame
            .semantic_ids
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        let depth = frame
            .depth
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>();
        println!(
            "{}{{\"name\":\"{}\",\"width\":{},\"height\":{},\"clock_millis\":{},\"maximum_rgba_channel_difference\":1,\"digests\":{{\"rgba8\":\"{}\",\"semantic_u32le\":\"{}\",\"depth_i16le\":\"{}\",\"coverage_u8\":\"{}\"}}}}",
            if index == 0 { "" } else { "," },
            name,
            width,
            height,
            scene.clock_millis(),
            digest(&frame.rgba8),
            digest(&semantic),
            digest(&depth),
            digest(&frame.coverage),
        );
    }
    println!("]}}");
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}
