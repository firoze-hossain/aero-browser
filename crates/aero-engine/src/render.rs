//! # Aero Display List & Software Rasterizer
//!
//! Converts layout box trees into hardware-independent display lists, then rasterizes
//! rectangles, rounded corners, borders, gradients, and typography directly to 32-bit pixel buffers.

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

use crate::css::{Color, Value};
use crate::font::{blend_pixel, render_text};
use crate::layout::{BoxType, LayoutBox, Rect};
use crate::style::StyledNode;

/// Atomic drawing operation in a browser frame.
#[derive(Debug, Clone)]
pub enum DisplayItem {
    /// Solid filled rectangle.
    SolidColor { rect: Rect, color: Color },
    /// Rounded corner rectangle (e.g. buttons, card elements).
    RoundedRect { rect: Rect, color: Color, radius: f32 },
    /// Outlined border rectangle.
    Border { rect: Rect, color: Color, width: f32, radius: f32 },
    /// Rendered text run.
    TextRun {
        text: String,
        x: f32,
        y: f32,
        color: Color,
        font_size: f32,
        bold: bool,
    },
    /// Linear vertical gradient fill.
    LinearGradient {
        rect: Rect,
        top_color: Color,
        bottom_color: Color,
    },
}

/// Ordered list of display commands for a page frame.
#[derive(Debug, Clone, Default)]
pub struct DisplayList {
    pub items: Vec<DisplayItem>,
}

impl DisplayList {
    pub fn new() -> Self {
        DisplayList { items: Vec::new() }
    }

    pub fn push(&mut self, item: DisplayItem) {
        self.items.push(item);
    }

    /// Rasterizes all display items into the target 32-bit XRGB pixel surface with scrolling offset.
    pub fn paint(&self, surface: &mut [u32], surface_w: usize, surface_h: usize, scroll_y: f32) {
        for item in &self.items {
            match item {
                DisplayItem::SolidColor { rect, color } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    fill_rect(surface, surface_w, surface_h, rx, ry, rw, rh, *color);
                }
                DisplayItem::RoundedRect { rect, color, radius } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    fill_rounded_rect(surface, surface_w, surface_h, rx, ry, rw, rh, *radius as i32, *color);
                }
                DisplayItem::Border { rect, color, width, radius } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    draw_border(surface, surface_w, surface_h, rx, ry, rw, rh, *width as i32, *radius as i32, *color);
                }
                DisplayItem::TextRun { text, x, y, color, font_size, bold } => {
                    let px = x.round() as i32;
                    let py = (y - scroll_y).round() as i32;

                    render_text(surface, surface_w, surface_h, px, py, text, *color, *font_size, *bold);
                }
                DisplayItem::LinearGradient { rect, top_color, bottom_color } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    fill_gradient(surface, surface_w, surface_h, rx, ry, rw, rh, *top_color, *bottom_color);
                }
            }
        }
    }
}

/// Generates a display list from the computed layout box tree.
pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = DisplayList::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    render_content(list, layout_box);

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(styled) = layout_box.get_styled_node() {
        if let Some(bg_color) = styled.background_color() {
            let radius = styled
                .value("border-radius")
                .map(|v| v.to_px())
                .unwrap_or(0.0);

            let border_box = layout_box.dimensions.border_box();
            if radius > 0.0 {
                list.push(DisplayItem::RoundedRect {
                    rect: border_box,
                    color: bg_color,
                    radius,
                });
            } else {
                list.push(DisplayItem::SolidColor {
                    rect: border_box,
                    color: bg_color,
                });
            }
        }
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(styled) = layout_box.get_styled_node() {
        let border_width = layout_box.dimensions.border.left;
        if border_width > 0.0 {
            let border_color = styled
                .value("border-color")
                .and_then(|v| v.to_color())
                .unwrap_or(Color::BORDER_COLOR);

            let radius = styled
                .value("border-radius")
                .map(|v| v.to_px())
                .unwrap_or(0.0);

            list.push(DisplayItem::Border {
                rect: layout_box.dimensions.border_box(),
                color: border_color,
                width: border_width,
                radius,
            });
        }
    }
}

fn render_content(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(styled) = layout_box.get_styled_node() {
        let color = styled.color();
        let font_size = styled
            .value("font-size")
            .map(|v| v.to_px())
            .unwrap_or(16.0);
        let bold = styled
            .value("font-weight")
            .map_or(false, |v| v == &Value::Keyword("bold".to_owned()));

        // If this node has direct text content
        if let crate::dom::NodeType::Text(ref text) = styled.node.node_type {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                list.push(DisplayItem::TextRun {
                    text: trimmed.to_owned(),
                    x: layout_box.dimensions.content.x,
                    y: layout_box.dimensions.content.y,
                    color,
                    font_size,
                    bold,
                });
            }
        } else if let Some(tag) = styled.node.tag_name() {
            if tag == "hr" {
                list.push(DisplayItem::SolidColor {
                    rect: Rect::new(
                        layout_box.dimensions.content.x,
                        layout_box.dimensions.content.y,
                        layout_box.dimensions.content.width,
                        1.0,
                    ),
                    color: Color::BORDER_COLOR,
                });
            } else if tag == "button" {
                // Render button label text if children have text
                let btn_text = styled.node.text_content();
                if !btn_text.trim().is_empty() {
                    list.push(DisplayItem::TextRun {
                        text: btn_text.trim().to_owned(),
                        x: layout_box.dimensions.content.x + 8.0,
                        y: layout_box.dimensions.content.y + 8.0,
                        color: Color::WHITE,
                        font_size: 14.0,
                        bold: true,
                    });
                }
            }
        }
    }
}

fn fill_rect(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32, h: i32, color: Color) {
    let x0 = x.clamp(0, sw as i32) as usize;
    let y0 = y.clamp(0, sh as i32) as usize;
    let x1 = (x + w).clamp(0, sw as i32) as usize;
    let y1 = (y + h).clamp(0, sh as i32) as usize;

    let xrgb = color.to_xrgb();
    for cy in y0..y1 {
        let row_start = cy * sw;
        for cx in x0..x1 {
            surface[row_start + cx] = xrgb;
        }
    }
}

fn fill_rounded_rect(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    r: i32,
    color: Color,
) {
    if r <= 0 {
        fill_rect(surface, sw, sh, x, y, w, h, color);
        return;
    }

    let r_clamped = r.min(w / 2).min(h / 2);
    let xrgb = color.to_xrgb();

    for cy in 0..h {
        let py = y + cy;
        if py < 0 || py >= sh as i32 {
            continue;
        }

        for cx in 0..w {
            let px = x + cx;
            if px < 0 || px >= sw as i32 {
                continue;
            }

            // Check corner radii
            let in_top_left = cx < r_clamped && cy < r_clamped;
            let in_top_right = cx >= w - r_clamped && cy < r_clamped;
            let in_bottom_left = cx < r_clamped && cy >= h - r_clamped;
            let in_bottom_right = cx >= w - r_clamped && cy >= h - r_clamped;

            let mut inside = true;
            if in_top_left {
                let dx = r_clamped - cx - 1;
                let dy = r_clamped - cy - 1;
                inside = dx * dx + dy * dy <= r_clamped * r_clamped;
            } else if in_top_right {
                let dx = cx - (w - r_clamped);
                let dy = r_clamped - cy - 1;
                inside = dx * dx + dy * dy <= r_clamped * r_clamped;
            } else if in_bottom_left {
                let dx = r_clamped - cx - 1;
                let dy = cy - (h - r_clamped);
                inside = dx * dx + dy * dy <= r_clamped * r_clamped;
            } else if in_bottom_right {
                let dx = cx - (w - r_clamped);
                let dy = cy - (h - r_clamped);
                inside = dx * dx + dy * dy <= r_clamped * r_clamped;
            }

            if inside {
                surface[py as usize * sw + px as usize] = xrgb;
            }
        }
    }
}

fn draw_border(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    b_width: i32,
    r: i32,
    color: Color,
) {
    if b_width <= 0 {
        return;
    }
    // Top border
    fill_rect(surface, sw, sh, x, y, w, b_width, color);
    // Bottom border
    fill_rect(surface, sw, sh, x, y + h - b_width, w, b_width, color);
    // Left border
    fill_rect(surface, sw, sh, x, y, b_width, h, color);
    // Right border
    fill_rect(surface, sw, sh, x + w - b_width, y, b_width, h, color);
}

fn fill_gradient(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    top_color: Color,
    bottom_color: Color,
) {
    if h <= 0 {
        return;
    }

    for cy in 0..h {
        let py = y + cy;
        if py < 0 || py >= sh as i32 {
            continue;
        }

        let t = cy as f32 / h as f32;
        let r = ((top_color.r as f32 * (1.0 - t)) + (bottom_color.r as f32 * t)) as u8;
        let g = ((top_color.g as f32 * (1.0 - t)) + (bottom_color.g as f32 * t)) as u8;
        let b = ((top_color.b as f32 * (1.0 - t)) + (bottom_color.b as f32 * t)) as u8;
        let xrgb = Color::rgb(r, g, b).to_xrgb();

        let row_start = py as usize * sw;
        let x0 = x.clamp(0, sw as i32) as usize;
        let x1 = (x + w).clamp(0, sw as i32) as usize;
        for cx in x0..x1 {
            surface[row_start + cx] = xrgb;
        }
    }
}
