use std::time::Instant;

use minifb::{Key, Window, WindowOptions};
fn rgb_to_hex(r: u32, g: u32, b: u32) -> u32 {
    (r << 16) | (g << 8) | b
}

fn lerp(a: u32, b: u32, t: f32) -> u32 {
    (a as f32 * t + b as f32 * (1.0 - t)) as u32
}
pub fn display_image(image_data: Vec<Vec<(u8, u8, u8, u8)>>) {
    let height = image_data.len();
    let width = image_data[0].len();
    let grid_size = 10;

    // Create a buffer for the image with grid background
    let mut buffer: Vec<u32> = vec![0; width * height];

    for y in 0..height {
        for x in 0..width {
            let (r, g, b, a) = image_data[y][x];

            // Determine if this pixel is part of the grid pattern
            let is_grid = ((x / grid_size) % 2 == 0 && (y / grid_size) % 2 == 0)
                || ((x / grid_size) % 2 == 1 && (y / grid_size) % 2 == 1);

            let grid_color = if is_grid { 0xCCCCCC } else { 0xFFFFFF };

            // Pack RGBA into a single u32 value, considering transparency
            let pixel = if a < 255 {
                let bg_r = (grid_color >> 16) & 0xFF;
                let bg_g = (grid_color >> 8) & 0xFF;
                let bg_b = grid_color & 0xFF;

                let fg_r = r as u32;
                let fg_g = g as u32;
                let fg_b = b as u32;

                let alpha = a as f32 / 255.0;

                let final_r = lerp(fg_r, bg_r, alpha);
                let final_g = lerp(fg_g, bg_g, alpha);
                let final_b = lerp(fg_b, bg_b, alpha);

                rgb_to_hex(final_r, final_g, final_b)
            } else {
                rgb_to_hex(r as u32, g as u32, b as u32)
            };
            buffer[y * width + x] = pixel;
        }
    }

    // Create a window to display the image
    let mut window = Window::new("Image Display", width, height, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let start_time = Instant::now();
    // Display the image
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width, height).unwrap();
        if (Instant::now() - start_time).as_secs_f32() > 1.0 {
            break;
        }
    }
}
