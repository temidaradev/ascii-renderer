use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::env;
use std::io::{Write, stdout};

mod ascii;
mod gui;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--gui") {
        if let Err(e) = gui::run() {
            eprintln!("Failed to start GUI: {}", e);
        }
        return;
    }

    let image_path = args
        .iter()
        .skip(1)
        .find(|arg| *arg != "--gui")
        .map(String::as_str)
        .unwrap_or("cover.png");

    let img_bytes = std::fs::read(image_path).expect("Failed to read image file");
    let img = image::load_from_memory(&img_bytes).expect("Failed to open image");

    let (term_w, term_h) = terminal::size().expect("Failed to get terminal size");

    let max_w = term_w as u32;
    let max_h = (term_h as u32).saturating_sub(1);

    if max_w == 0 || max_h == 0 {
        return;
    }

    let art = ascii::generate_ascii_art(&img, max_w, max_h);

    if art.width == 0 || art.height == 0 {
        return;
    }

    let mut out = stdout();

    for ay in 0..art.height {
        for ax in 0..art.width {
            let idx = (ay * art.width + ax) as usize;
            let c = &art.characters[idx];

            queue!(
                out,
                SetForegroundColor(Color::Rgb {
                    r: c.r,
                    g: c.g,
                    b: c.b
                }),
                Print(c.character)
            )
            .unwrap();
        }

        queue!(out, ResetColor, Print("\n")).unwrap();
    }

    out.flush().unwrap();
}
