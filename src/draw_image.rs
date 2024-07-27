use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

fn scale_pixels(
    pixels: Vec<Vec<(u8, u8, u8, u8)>>,
    scale_factor: usize,
) -> Vec<Vec<(u8, u8, u8, u8)>> {
    let original_size = pixels.len();
    let new_size = original_size * scale_factor;
    let mut scaled_pixels = vec![vec![(0, 0, 0, 0); new_size]; new_size];

    for (i, row) in pixels.iter().enumerate() {
        for (j, &pixel) in row.iter().enumerate() {
            for x in 0..scale_factor {
                for y in 0..scale_factor {
                    scaled_pixels[i * scale_factor + x][j * scale_factor + y] = pixel;
                }
            }
        }
    }

    scaled_pixels
}

fn create_transparency_background(size: usize, tile_size: usize) -> Vec<Vec<(u8, u8, u8, u8)>> {
    let mut background = vec![vec![(0, 0, 0, 255); size]; size];
    for i in 0..size {
        for j in 0..size {
            let is_light = ((i / tile_size) % 2 == 0) != ((j / tile_size) % 2 == 0);
            background[i][j] = if is_light {
                (192, 192, 192, 255) // light gray
            } else {
                (128, 128, 128, 255) // dark gray
            };
        }
    }
    background
}

pub fn display_pixels(
    pixels: Vec<Vec<(u8, u8, u8, u8)>>,
    scale_factor: usize,
) -> Result<(), String> {
    let original_size = pixels.len();
    let scaled_size = original_size * scale_factor;
    let tile_size = 10; // Size of each checkered tile for transparency background

    // Create scaled pixel data
    let scaled_pixels = scale_pixels(pixels, scale_factor);

    // Create transparency background
    let background = create_transparency_background(scaled_size, tile_size);

    let window_size = scaled_size as u32;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Display Pixels", window_size, window_size)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Create texture for transparency background
    let mut background_texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            scaled_size as u32,
            scaled_size as u32,
        )
        .map_err(|e| e.to_string())?;

    let mut flat_background: Vec<u8> = Vec::new();
    for row in background {
        for (r, g, b, a) in row {
            flat_background.push(a);
            flat_background.push(b);
            flat_background.push(g);
            flat_background.push(r);
        }
    }

    background_texture
        .update(None, &flat_background, scaled_size * 4)
        .map_err(|e| e.to_string())?;

    // Create texture for pixel data
    let mut pixels_texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            scaled_size as u32,
            scaled_size as u32,
        )
        .map_err(|e| e.to_string())?;

    let mut flat_pixels: Vec<u8> = Vec::new();
    for row in scaled_pixels {
        for (r, g, b, a) in row {
            flat_pixels.push(a);
            flat_pixels.push(b);
            flat_pixels.push(g);
            flat_pixels.push(r);
        }
    }

    pixels_texture
        .update(None, &flat_pixels, scaled_size * 4)
        .map_err(|e| e.to_string())?;
    pixels_texture.set_blend_mode(BlendMode::Blend);

    canvas.clear();
    canvas.copy(
        &background_texture,
        None,
        Some(Rect::new(0, 0, window_size, window_size)),
    )?;
    canvas.copy(
        &pixels_texture,
        None,
        Some(Rect::new(0, 0, window_size, window_size)),
    )?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
