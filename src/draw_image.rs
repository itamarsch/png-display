use minifb::{Key, Window, WindowOptions};

pub fn display_image(image_data: Vec<Vec<(u8, u8, u8, u8)>>) {
    let height = image_data.len();
    let width = image_data[0].len();

    // Flatten the 2D Vec into a 1D Vec and convert to u32 pixels
    let mut buffer: Vec<u32> = Vec::with_capacity(width * height);
    for row in image_data.iter() {
        for &(r, g, b, a) in row.iter() {
            // Pack RGBA into a single u32 value
            let pixel = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            buffer.push(pixel);
        }
    }

    // Create a window to display the image
    let mut window = Window::new("Image Display", width, height, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Display the image
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
