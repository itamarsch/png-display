use anyhow::Context;
use draw_image::display_image;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::ExitCode;
use std::time::Duration;

pub mod ancillary_chunks;
pub mod chunk;
mod color_type;
pub mod draw_image;
pub mod filter_apply;
pub mod ihdr;
pub mod plte;
pub mod png_parser;
pub mod run_n;

fn main_inner() -> anyhow::Result<()> {
    let filename = std::env::args().nth(1).context("No args passed")?;

    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let png = png_parser::Png::new(&buf)?;

    let pixels = png.get_pixels()?;
    png.print_ancillary();

    let bg = png.other_chunks.get_background();
    let gama = png.other_chunks.get_gama();

    display_image(
        pixels,
        900.0 / png.ihdr.height as f32,
        Some(Duration::from_secs_f32(0.3f32)),
        bg,
        gama,
    )
}

fn main() -> ExitCode {
    env::set_var("RUST_LIB_BACKTRACE", "1");

    let res = main_inner();
    if let Err(err) = &res {
        println!("Failed: {}", err);
        print_my_backtrace(err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_my_backtrace(err: &anyhow::Error) {
    let bt = btparse_stable::deserialize(err.backtrace());
    if let Ok(bt) = bt {
        for frame in bt.frames.iter().filter(|f| {
            f.file.as_ref().is_some_and(|file| file.starts_with('.')) && f.line.is_some()
        }) {
            println!(
                "{} {}:{}",
                frame.function,
                frame.file.as_ref().unwrap(),
                frame.line.unwrap()
            );
        }
    }
}
