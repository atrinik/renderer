# GPU lifecycle and ownership

One `WgpuRenderer` owns one adapter, device, queue, and pipeline. Every offscreen
frame uses the same render path: validate scene/target agreement, create a
bounded color target and vertex buffer, submit once, wait for bounded readback,
strip WebGPU row padding, and produce exact semantic planes. Resize validates
the new total pixel budget and increments recovery metrics.

The SDL bridge borrows an existing window or creates one proof window. A native
surface cannot outlive that borrow. Timeout and occlusion skip a frame; outdated
surfaces are reconfigured; lost/device failures are typed for owner-driven
recreation. The bridge never owns the event loop. Window and embedded consumers
must call it on SDL's video thread.

Linux runtime proof uses Vulkan (including Mesa software Vulkan in headless CI).
Native Windows uses D3D12. CI cross-checks the non-SDL renderer crates for the
Windows GNU target; display presentation remains a documented manual test
because hosted CI has no native interactive desktop.

The sole unsafe assertion is private to `raw_handle_bridge`: an ephemeral
wrapper is declared `Send + Sync` only to synchronously copy immutable raw SDL
handles into wgpu. It is never stored, sent, or used for an SDL operation, and
the returned surface retains the original window lifetime.
