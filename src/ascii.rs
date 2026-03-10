use image::{DynamicImage, GenericImageView, imageops::FilterType};
use rayon::prelude::*;

pub const ASCII: &[char] = &[
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-',
    '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v',
    'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
    'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$', '█',
];

#[inline]
pub fn pixel_to_char(r: u8, g: u8, b: u8) -> char {
    let brightness = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    let index = ((brightness / 255.0) * (ASCII.len() - 1) as f32)
        .clamp(0.0, (ASCII.len() - 1) as f32) as usize;

    ASCII[index]
}

pub struct AsciiCharacter {
    pub character: char,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct AsciiArt {
    pub width: u32,
    pub height: u32,
    pub characters: Vec<AsciiCharacter>,
}

pub fn generate_ascii_art(img: &DynamicImage, max_w: u32, max_h: u32) -> AsciiArt {
    generate_ascii_art_with_aspect(img, max_w, max_h, 0.5_f32)
}

pub fn generate_ascii_art_with_aspect(
    img: &DynamicImage,
    max_w: u32,
    max_h: u32,
    char_aspect: f32,
) -> AsciiArt {
    let (img_w, img_h) = img.dimensions();

    if max_w == 0 || max_h == 0 || img_w == 0 || img_h == 0 {
        return AsciiArt {
            width: 0,
            height: 0,
            characters: Vec::new(),
        };
    }

    let h_from_w = (max_w as f32 * (img_h as f32 / img_w as f32) * char_aspect) as u32;

    let (ascii_w, ascii_h) = if h_from_w <= max_h {
        (max_w, h_from_w)
    } else {
        let w_from_h = (max_h as f32 * (img_w as f32 / img_h as f32) / char_aspect) as u32;
        (w_from_h.min(max_w), max_h)
    };

    if ascii_w == 0 || ascii_h == 0 {
        return AsciiArt {
            width: 0,
            height: 0,
            characters: Vec::new(),
        };
    }

    let resized_img = img.resize_exact(ascii_w, ascii_h, FilterType::Triangle);
    let rgba_img = resized_img.to_rgba8();

    let characters: Vec<AsciiCharacter> = rgba_img
        .as_raw()
        .par_chunks_exact(4)
        .map(|p| {
            let (r, g, b) = (p[0], p[1], p[2]);
            let character = pixel_to_char(r, g, b);

            AsciiCharacter { character, r, g, b }
        })
        .collect();

    AsciiArt {
        width: ascii_w,
        height: ascii_h,
        characters,
    }
}
