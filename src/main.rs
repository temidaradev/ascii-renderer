use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use image::{GenericImageView, imageops::FilterType};
use std::env;
use std::io::{Write, stdout};

const ASCII: &str = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";

fn pixel_to_char(r: u8, g: u8, b: u8) -> char {
    let brightness = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    let index = ((brightness / 255.0) * (ASCII.len() - 1) as f32)
        .clamp(0.0, (ASCII.len() - 1) as f32) as usize;
    ASCII.chars().nth(index).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = args.get(1).map(String::as_str).unwrap_or("cover.png");

    let img = image::open(image_path).expect("Failed to open image");
    let (img_w, img_h) = img.dimensions();

    let (term_w, term_h) = terminal::size().expect("Failed to get terminal size");

    let char_aspect = 0.5_f32;
    let max_w = term_w as u32;
    let max_h = (term_h as u32).saturating_sub(1);

    if max_w == 0 || max_h == 0 {
        return;
    }

    let h_from_w = (max_w as f32 * (img_h as f32 / img_w as f32) * char_aspect) as u32;

    let (ascii_w, ascii_h) = if h_from_w <= max_h {
        (max_w, h_from_w)
    } else {
        let w_from_h = (max_h as f32 * (img_w as f32 / img_h as f32) / char_aspect) as u32;
        (w_from_h.min(max_w), max_h)
    };

    let resized_img = img.resize_exact(ascii_w, ascii_h, FilterType::Lanczos3);

    let mut out = stdout();

    for ay in 0..ascii_h {
        for ax in 0..ascii_w {
            let p = resized_img.get_pixel(ax, ay);
            let (r, g, b) = (p[0], p[1], p[2]);

            let c = pixel_to_char(r, g, b);

            queue!(out, SetForegroundColor(Color::Rgb { r, g, b }), Print(c)).unwrap();
        }

        queue!(out, ResetColor, Print("\n")).unwrap();
    }

    out.flush().unwrap();
}
