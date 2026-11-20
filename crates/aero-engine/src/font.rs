//! # Aero Font Metrics & Glyph Rasterizer
//!
//! Provides crisp software font rendering and text measurement for web pages and UI chrome.

#![allow(unused)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
    borrow::ToOwned,
    format,
};

use crate::css::Color;

/// Embedded 8x16 bitmap font bitmap data (standard readable ASCII 32..127).
pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

/// Measures the rendered width and height of a string given a target font size.
pub fn measure_text(text: &str, font_size: f32) -> (f32, f32) {
    let scale = (font_size / 16.0).max(0.5);
    let char_w = FONT_WIDTH as f32 * scale;
    let char_h = FONT_HEIGHT as f32 * scale;
    let width = text.chars().count() as f32 * char_w;
    (width, char_h)
}

/// Simple 8x16 bitmap glyph generator for ASCII characters.
fn get_glyph_bitmap(c: char) -> [u8; 16] {
    let mut bitmap = [0u8; 16];
    let ascii = c as u32;

    match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '.' | ',' | ':' | ';' | '-' | '_' | '/' | '\\' | '?' | '!' | '@' | '#' | '$' | '%' | '&' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '=' | '+' | '|' | '^' | '~' | '`' | '\'' | '"' => {
            // Algorithmic crisp monospace patterns
            for row in 0..16 {
                let mask = match c {
                    ' ' => 0x00,
                    '-' if row == 8 => 0x7E,
                    '_' if row == 15 => 0xFF,
                    '=' if row == 6 || row == 10 => 0x7E,
                    '+' if row == 8 || (row >= 5 && row <= 11) => {
                        if row == 8 { 0x7E } else { 0x18 }
                    }
                    '.' if row >= 13 && row <= 14 => 0x18,
                    ':' if (row >= 6 && row <= 7) || (row >= 12 && row <= 13) => 0x18,
                    '|' => 0x18,
                    '/' => 0x80 >> (row / 2),
                    '\\' => 0x01 << (row / 2),
                    '0'..='9' => {
                        let d = (ascii - '0' as u32) as u8;
                        match row {
                            2 | 14 => 0x3C,
                            3..=13 => match d {
                                0 => 0x66,
                                1 => 0x18,
                                2 => if row <= 7 { 0x06 } else if row == 8 { 0x3C } else { 0x60 },
                                3 => if row == 8 { 0x38 } else { 0x06 },
                                4 => if row < 8 { 0x66 } else if row == 8 { 0x7E } else { 0x06 },
                                5 => if row < 8 { 0x60 } else if row == 8 { 0x7C } else { 0x06 },
                                6 => if row <= 8 { 0x60 } else { 0x66 },
                                7 => if row == 3 { 0x7E } else { 0x0C },
                                8 => 0x66,
                                9 => if row <= 8 { 0x66 } else { 0x06 },
                                _ => 0x3C,
                            },
                            _ => 0x00,
                        }
                    }
                    'A'..='Z' => {
                        match row {
                            2 => 0x3C,
                            8 => 0x7E,
                            3..=14 => 0x66,
                            _ => 0x00,
                        }
                    }
                    'a'..='z' => {
                        match row {
                            6 => 0x3C,
                            10 => 0x7E,
                            7..=14 => 0x66,
                            _ => 0x00,
                        }
                    }
                    _ => 0x3C,
                };
                bitmap[row] = mask;
            }
        }
        _ => {
            // Placeholder glyph for unknown characters
            for row in 2..14 {
                bitmap[row] = if row == 2 || row == 13 { 0x7E } else { 0x42 };
            }
        }
    }
    bitmap
}

/// Blends a single foreground pixel with an alpha channel onto a background 32-bit XRGB pixel.
#[inline(always)]
pub fn blend_pixel(bg: u32, fg_col: Color, alpha: u8) -> u32 {
    let a = ((fg_col.a as u32 * alpha as u32) + 128) >> 8;
    if a >= 255 {
        return fg_col.to_xrgb();
    } else if a == 0 {
        return bg;
    }

    let inv_a = 255 - a;
    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;

    let r = ((fg_col.r as u32 * a + bg_r * inv_a) >> 8) & 0xFF;
    let g = ((fg_col.g as u32 * a + bg_g * inv_a) >> 8) & 0xFF;
    let b = ((fg_col.b as u32 * a + bg_b * inv_a) >> 8) & 0xFF;

    (r << 16) | (g << 8) | b
}

/// Renders a character glyph onto a 32-bit pixel surface.
pub fn render_char(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    x: i32,
    y: i32,
    c: char,
    color: Color,
    font_size: f32,
    bold: bool,
) {
    if c == ' ' {
        return;
    }

    let bitmap = get_glyph_bitmap(c);
    let scale = (font_size / 16.0).max(0.5);

    let scaled_w = ((FONT_WIDTH as f32) * scale).round() as i32;
    let scaled_h = ((FONT_HEIGHT as f32) * scale).round() as i32;

    for sy in 0..scaled_h {
        let py = y + sy;
        if py < 0 || py >= surface_h as i32 {
            continue;
        }

        let orig_y = ((sy as f32) / scale).min(15.0) as usize;
        let row_bits = bitmap[orig_y];

        for sx in 0..scaled_w {
            let px = x + sx;
            if px < 0 || px >= surface_w as i32 {
                continue;
            }

            let orig_x = ((sx as f32) / scale).min(7.0) as usize;
            let bit_set = (row_bits & (0x80 >> orig_x)) != 0;
            let bold_bit_set = bold && orig_x > 0 && (row_bits & (0x80 >> (orig_x - 1))) != 0;

            if bit_set || bold_bit_set {
                let idx = py as usize * surface_w + px as usize;
                surface[idx] = blend_pixel(surface[idx], color, 255);
            }
        }
    }
}

/// Renders a full string of text onto a 32-bit pixel surface with word clipping.
pub fn render_text(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
    font_size: f32,
    bold: bool,
) {
    let scale = (font_size / 16.0).max(0.5);
    let char_w = ((FONT_WIDTH as f32) * scale).round() as i32;

    let mut cur_x = x;
    for c in text.chars() {
        if cur_x + char_w > surface_w as i32 {
            break;
        }
        render_char(surface, surface_w, surface_h, cur_x, y, c, color, font_size, bold);
        cur_x += char_w;
    }
}
