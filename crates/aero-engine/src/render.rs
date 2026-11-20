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
    RoundedRect {
        rect: Rect,
        color: Color,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    },
    /// Outlined border rectangle.
    Border {
        rect: Rect,
        color: Color,
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    },
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
                DisplayItem::RoundedRect { rect, color, tl, tr, br, bl } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    fill_rounded_rect(
                        surface,
                        surface_w,
                        surface_h,
                        rx,
                        ry,
                        rw,
                        rh,
                        *tl as i32,
                        *tr as i32,
                        *br as i32,
                        *bl as i32,
                        *color,
                    );
                }
                DisplayItem::Border { rect, color, top, right, bottom, left, tl, tr, br, bl } => {
                    let rx = (rect.x.round() as i32).max(0);
                    let ry = ((rect.y - scroll_y).round() as i32);
                    let rw = (rect.width.round() as i32).max(0);
                    let rh = (rect.height.round() as i32).max(0);

                    if *tl > 0.0 || *tr > 0.0 || *br > 0.0 || *bl > 0.0 {
                        draw_rounded_border(
                            surface,
                            surface_w,
                            surface_h,
                            rx,
                            ry,
                            rw,
                            rh,
                            *top as i32,
                            *right as i32,
                            *bottom as i32,
                            *left as i32,
                            *tl as i32,
                            *tr as i32,
                            *br as i32,
                            *bl as i32,
                            *color,
                        );
                    } else {
                        if *top > 0.0 {
                            fill_rect(surface, surface_w, surface_h, rx, ry, rw, *top as i32, *color);
                        }
                        if *bottom > 0.0 {
                            fill_rect(surface, surface_w, surface_h, rx, ry + rh - (*bottom as i32), rw, *bottom as i32, *color);
                        }
                        if *left > 0.0 {
                            fill_rect(surface, surface_w, surface_h, rx, ry, *left as i32, rh, *color);
                        }
                        if *right > 0.0 {
                            fill_rect(surface, surface_w, surface_h, rx + rw - (*right as i32), ry, *right as i32, rh, *color);
                        }
                    }
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
            let base_radius = styled
                .value("border-radius")
                .map(|v| v.to_px())
                .unwrap_or(0.0);

            let tl = styled.value("border-top-left-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let tr = styled.value("border-top-right-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let br = styled.value("border-bottom-right-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let bl = styled.value("border-bottom-left-radius").map(|v| v.to_px()).unwrap_or(base_radius);

            let border_box = layout_box.dimensions.border_box();
            if tl > 0.0 || tr > 0.0 || br > 0.0 || bl > 0.0 {
                list.push(DisplayItem::RoundedRect {
                    rect: border_box,
                    color: bg_color,
                    tl,
                    tr,
                    br,
                    bl,
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
        let border = layout_box.dimensions.border;
        if border.top > 0.0 || border.bottom > 0.0 || border.left > 0.0 || border.right > 0.0 {
            let border_color = styled
                .value("border-color")
                .or_else(|| styled.value("border-top-color"))
                .or_else(|| styled.value("border-bottom-color"))
                .or_else(|| styled.value("border-left-color"))
                .or_else(|| styled.value("border-right-color"))
                .and_then(|v| v.to_color())
                .unwrap_or(Color::BORDER_COLOR);

            let base_radius = styled
                .value("border-radius")
                .map(|v| v.to_px())
                .unwrap_or(0.0);

            let tl = styled.value("border-top-left-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let tr = styled.value("border-top-right-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let br = styled.value("border-bottom-right-radius").map(|v| v.to_px()).unwrap_or(base_radius);
            let bl = styled.value("border-bottom-left-radius").map(|v| v.to_px()).unwrap_or(base_radius);

            list.push(DisplayItem::Border {
                rect: layout_box.dimensions.border_box(),
                color: border_color,
                top: border.top,
                right: border.right,
                bottom: border.bottom,
                left: border.left,
                tl,
                tr,
                br,
                bl,
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
                let is_center = styled.lookup("text-align", "left") == crate::css::Value::Keyword("center".to_owned());
                let text_w = crate::font::measure_text(trimmed, font_size).0;
                let text_x = if is_center {
                    layout_box.dimensions.content.x + (layout_box.dimensions.content.width - text_w).max(0.0) / 2.0
                } else {
                    layout_box.dimensions.content.x
                };

                list.push(DisplayItem::TextRun {
                    text: trimmed.to_owned(),
                    x: text_x,
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

fn is_inside_rounded_corners(cx: i32, cy: i32, w: i32, h: i32, tl: i32, tr: i32, br: i32, bl: i32) -> bool {
    if cx < 0 || cy < 0 || cx >= w || cy >= h {
        return false;
    }
    if tl > 0 && cx < tl && cy < tl {
        let dx = tl - cx - 1;
        let dy = tl - cy - 1;
        dx * dx + dy * dy <= tl * tl
    } else if tr > 0 && cx >= w - tr && cy < tr {
        let dx = cx - (w - tr);
        let dy = tr - cy - 1;
        dx * dx + dy * dy <= tr * tr
    } else if bl > 0 && cx < bl && cy >= h - bl {
        let dx = bl - cx - 1;
        let dy = cy - (h - bl);
        dx * dx + dy * dy <= bl * bl
    } else if br > 0 && cx >= w - br && cy >= h - br {
        let dx = cx - (w - br);
        let dy = cy - (h - br);
        dx * dx + dy * dy <= br * br
    } else {
        true
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
    tl: i32,
    tr: i32,
    br: i32,
    bl: i32,
    color: Color,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    if tl <= 0 && tr <= 0 && br <= 0 && bl <= 0 {
        fill_rect(surface, sw, sh, x, y, w, h, color);
        return;
    }

    let max_r = (w / 2).min(h / 2);
    let tl = tl.min(max_r).max(0);
    let tr = tr.min(max_r).max(0);
    let br = br.min(max_r).max(0);
    let bl = bl.min(max_r).max(0);
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

            if is_inside_rounded_corners(cx, cy, w, h, tl, tr, br, bl) {
                surface[py as usize * sw + px as usize] = xrgb;
            }
        }
    }
}

fn draw_rounded_border(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    b_top: i32,
    b_right: i32,
    b_bottom: i32,
    b_left: i32,
    tl: i32,
    tr: i32,
    br: i32,
    bl: i32,
    color: Color,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    if b_top <= 0 && b_right <= 0 && b_bottom <= 0 && b_left <= 0 {
        return;
    }

    let max_r = (w / 2).min(h / 2);
    let tl_out = tl.min(max_r).max(0);
    let tr_out = tr.min(max_r).max(0);
    let br_out = br.min(max_r).max(0);
    let bl_out = bl.min(max_r).max(0);

    let inner_w = (w - b_left - b_right).max(0);
    let inner_h = (h - b_top - b_bottom).max(0);

    let tl_in = (tl_out - b_top.max(b_left)).max(0);
    let tr_in = (tr_out - b_top.max(b_right)).max(0);
    let br_in = (br_out - b_bottom.max(b_right)).max(0);
    let bl_in = (bl_out - b_bottom.max(b_left)).max(0);

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

            if !is_inside_rounded_corners(cx, cy, w, h, tl_out, tr_out, br_out, bl_out) {
                continue;
            }

            let in_inner = inner_w > 0
                && inner_h > 0
                && cx >= b_left
                && cx < w - b_right
                && cy >= b_top
                && cy < h - b_bottom
                && is_inside_rounded_corners(cx - b_left, cy - b_top, inner_w, inner_h, tl_in, tr_in, br_in, bl_in);

            if !in_inner {
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
