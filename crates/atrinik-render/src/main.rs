// Copyright 2026 The Atrinik Project
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

use atrinik_render_api::{
    BackendPreference, RenderRequest, Renderer, TargetDescriptor, TargetKind,
};
use atrinik_render_testkit::{FakeResourceProvider, synthetic_scene};
use atrinik_render_wgpu::WgpuRenderer;
use std::{env, io::Write, path::Path, process::ExitCode, sync::Arc};

#[cfg(unix)]
use std::fs::File;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("atrinik-render: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        Some("--version") => {
            println!("atrinik-render {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("probe") if arguments.next().is_none() => {
            let renderer = WgpuRenderer::new(target(16, 16))
                .map_err(|error| format!("GPU probe failed: {error}"))?;
            let adapter = renderer.adapter();
            println!(
                "adapter={} driver={} backend={} type={}",
                adapter.name, adapter.driver, adapter.backend, adapter.device_type
            );
            Ok(())
        }
        Some("offscreen") => {
            let output = arguments
                .next()
                .ok_or("usage: atrinik-render offscreen OUTPUT.rgba")?;
            if arguments.next().is_some() {
                return Err("usage: atrinik-render offscreen OUTPUT.rgba".to_owned());
            }
            render_offscreen(Path::new(&output))
        }
        _ => Err("usage: atrinik-render {--version|probe|offscreen OUTPUT.rgba}".to_owned()),
    }
}

fn render_offscreen(path: &Path) -> Result<(), String> {
    let scene = synthetic_scene(64, 64).map_err(|error| error.to_string())?;
    let mut renderer = WgpuRenderer::new(target(64, 64)).map_err(|error| error.to_string())?;
    let frame = renderer
        .render(RenderRequest {
            scene: &scene,
            resources: Arc::new(FakeResourceProvider::default()),
        })
        .map_err(|error| error.to_string())?;
    persist_new(path, &[b"ATRINIK-RGBA8\n64 64\n", &frame.rgba8])
}

fn persist_new(path: &Path, chunks: &[&[u8]]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| {
        format!(
            "cannot create temporary output in {}: {error}",
            parent.display()
        )
    })?;
    for chunk in chunks {
        temporary
            .write_all(chunk)
            .map_err(|error| format!("cannot write temporary output: {error}"))?;
    }
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| format!("cannot sync temporary output: {error}"))?;
    let output = temporary.persist_noclobber(path).map_err(|error| {
        format!(
            "cannot create {} without overwriting: {error}",
            path.display()
        )
    })?;
    output
        .sync_all()
        .map_err(|error| format!("cannot sync {}: {error}", path.display()))?;
    sync_parent(parent)?;
    Ok(())
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), String> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("cannot sync output directory {}: {error}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<(), String> {
    Ok(())
}

const fn target(width: u32, height: u32) -> TargetDescriptor {
    TargetDescriptor {
        kind: TargetKind::Offscreen,
        width,
        height,
        backend: BackendPreference::Automatic,
    }
}
