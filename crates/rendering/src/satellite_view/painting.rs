//! Pixel-painting helpers for the satellite view texture.

/// Paint a filled circle into the pixel buffer.
pub(crate) fn paint_circle(
    pixels: &mut [[u8; 4]],
    size: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    color: [u8; 4],
) {
    let r2 = radius * radius + 0.5; // slight expansion for anti-alias
    let min_x = ((cx - radius).floor() as isize).max(0) as usize;
    let max_x = ((cx + radius).ceil() as usize).min(size - 1);
    let min_y = ((cy - radius).floor() as isize).max(0) as usize;
    let max_y = ((cy + radius).ceil() as usize).min(size - 1);

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 - cx;
            let dy = py as f32 - cy;
            if dx * dx + dy * dy <= r2 {
                pixels[py * size + px] = color;
            }
        }
    }
}

/// Paint a single grid cell into the satellite texture.
pub(crate) fn paint_grid_cell(
    pixels: &mut [[u8; 4]],
    size: usize,
    scale_x: f32,
    scale_y: f32,
    grid_x: usize,
    grid_y: usize,
    color: [u8; 4],
) {
    let px_start = (grid_x as f32 / scale_x).floor() as usize;
    let py_start = (grid_y as f32 / scale_y).floor() as usize;
    let px_end = (((grid_x + 1) as f32) / scale_x).ceil() as usize;
    let py_end = (((grid_y + 1) as f32) / scale_y).ceil() as usize;

    let px_end = px_end.min(size);
    let py_end = py_end.min(size);

    for py in py_start..py_end {
        for px in px_start..px_end {
            let idx = py * size + px;
            if idx < pixels.len() {
                pixels[idx] = color;
            }
        }
    }
}
