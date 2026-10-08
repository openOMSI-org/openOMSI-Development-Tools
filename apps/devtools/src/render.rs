//! Headless rendering: lay out a page with egui and paint it with wgpu into an off-screen
//! texture, then save a PNG. No window is ever opened, so the app's pages can be checked on a
//! build server (and during development) without anything appearing on screen.

use std::path::Path;

use egui_wgpu::Renderer;
use egui_wgpu::wgpu;

use crate::state::{AppState, Page, Theme};

/// The pixel format the egui renderer and the PNG both use.
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Renders `page` at `width`x`height` physical pixels and writes a PNG to `out`.
pub fn screenshot(page: Page, theme: Theme, width: u32, height: u32, ppp: f32, out: &Path) -> Result<(), String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .map_err(|e| format!("no GPU adapter for offscreen rendering: {e}"))?;
    let (device, queue) = pollster::block_on(
        adapter.request_device(&wgpu::DeviceDescriptor { label: Some("devtools-offscreen"), ..Default::default() }),
    )
    .map_err(|e| format!("cannot create a device: {e}"))?;

    // Lay the page out. Two passes so galley sizes and auto-sized widgets settle.
    let ctx = egui::Context::default();
    let mut state = AppState::demo();
    state.page = page;
    state.theme = theme;
    let raw = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(width as f32 / ppp, height as f32 / ppp),
        )),
        ..Default::default()
    };
    ctx.set_pixels_per_point(ppp);
    // Two layout passes let auto-sized widgets settle. The texture deltas from BOTH passes are
    // applied to the (fresh) renderer, so the first pass's full font-atlas upload is not lost.
    let first = ctx.run_ui(raw(), |ui| crate::ui::draw(ui, &mut state));
    let full = ctx.run_ui(raw(), |ui| crate::ui::draw(ui, &mut state));

    let paint_jobs = ctx.tessellate(full.shapes, full.pixels_per_point);
    let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [width, height], pixels_per_point: ppp };

    let mut renderer = Renderer::new(
        &device,
        FORMAT,
        egui_wgpu::RendererOptions {
            msaa_samples: 1,
            depth_stencil_format: None,
            dithering: true,
            ..Default::default()
        },
    );
    for (id, delta) in first.textures_delta.set.iter().chain(full.textures_delta.set.iter()) {
        renderer.update_texture(&device, &queue, *id, delta);
    }

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("devtools-target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("devtools") });
    let user_buffers = renderer.update_buffers(&device, &queue, &mut encoder, &paint_jobs, &screen);

    let bg = if theme == Theme::Dark {
        wgpu::Color { r: 0.082, g: 0.090, b: 0.110, a: 1.0 }
    } else {
        wgpu::Color { r: 0.96, g: 0.96, b: 0.97, a: 1.0 }
    };
    {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("devtools-egui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(bg), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let mut pass = pass.forget_lifetime();
        renderer.render(&mut pass, &paint_jobs, &screen);
    }
    for id in &full.textures_delta.free {
        renderer.free_texture(id);
    }

    // Copy the texture into a buffer with 256-aligned rows, then read it back.
    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("devtools-readback"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(user_buffers.into_iter().chain(std::iter::once(encoder.finish())));

    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("GPU poll failed: {e}"))?;
    let data = slice.get_mapped_range();

    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        pixels.extend_from_slice(&data[start..start + unpadded as usize]);
    }
    drop(data);
    readback.unmap();

    write_png(out, width, height, &pixels)?;
    Ok(())
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = std::fs::File::create(path).map_err(|e| format!("creating {}: {e}", path.display()))?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder
        .write_header()
        .map_err(|e| e.to_string())?
        .write_image_data(rgba)
        .map_err(|e| format!("writing PNG: {e}"))?;
    Ok(())
}
