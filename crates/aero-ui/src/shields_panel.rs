//! # Aero Interactive Shields Flyout Panel
//!
//! Brave-style drop-down privacy panel providing real-time tracker counters,
//! per-site shield toggles, and cryptographic HTTPS enforcement details.

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

use aero_engine::css::Color;
use aero_engine::font::render_text;
use aero_engine::shields::{ShieldConfig, ShieldEngine, ShieldStats};

/// Renders the interactive Brave-style Shields Flyout drop-down panel.
pub fn render_shields_flyout(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    x: i32,
    y: i32,
    domain: &str,
    shields: &ShieldEngine,
) {
    let panel_w = 320;
    let panel_h = 290;

    let px = x.min(surface_w as i32 - panel_w - 10).max(10);
    let py = y.min(surface_h as i32 - panel_h - 10).max(10);

    let is_up = shields.is_shields_up_for_domain(domain);

    // Panel Shadow & Background
    fill_rect(surface, surface_w, surface_h, px + 4, py + 4, panel_w, panel_h, Color::rgba(0, 0, 0, 160));
    fill_rounded_rect(surface, surface_w, surface_h, px, py, panel_w, panel_h, 8, Color::rgb(24, 30, 44));
    draw_border(surface, surface_w, surface_h, px, py, panel_w, panel_h, 1, 8, Color::rgb(51, 65, 85));

    // Top Header: Domain name + Shield badge
    let header_bg = if is_up { Color::rgb(30, 41, 59) } else { Color::rgb(40, 20, 20) };
    fill_rounded_rect(surface, surface_w, surface_h, px + 2, py + 2, panel_w - 4, 46, 6, header_bg);

    let title = format!("AERO SHIELDS");
    render_text(surface, surface_w, surface_h, px + 16, py + 10, &title, Color::BRAVE_ORANGE, 13.0, true);

    let site_name = if domain.is_empty() { "Local Browser Page" } else { domain };
    render_text(surface, surface_w, surface_h, px + 16, py + 26, site_name, Color::WHITE, 12.0, false);

    // Status Pill Toggle (Up / Down)
    let toggle_btn_w = 80;
    let toggle_btn_h = 24;
    let toggle_x = px + panel_w - toggle_btn_w - 14;
    let toggle_y = py + 12;
    let toggle_col = if is_up { Color::rgb(22, 163, 74) } else { Color::rgb(220, 38, 38) };
    fill_rounded_rect(surface, surface_w, surface_h, toggle_x, toggle_y, toggle_btn_w, toggle_btn_h, 12, toggle_col);
    let toggle_text = if is_up { "UP  ●" } else { "●  DOWN" };
    render_text(surface, surface_w, surface_h, toggle_x + 14, toggle_y + 5, toggle_text, Color::WHITE, 11.0, true);

    // Stats Grid
    let card_y = py + 56;
    fill_rounded_rect(surface, surface_w, surface_h, px + 12, card_y, panel_w - 24, 80, 6, Color::rgb(15, 23, 42));
    draw_border(surface, surface_w, surface_h, px + 12, card_y, panel_w - 24, 80, 1, 6, Color::rgb(51, 65, 85));

    // Stat 1: Trackers & ads blocked
    let t_str = format!("{}", shields.stats.trackers_blocked);
    render_text(surface, surface_w, surface_h, px + 24, card_y + 12, &t_str, Color::rgb(56, 189, 248), 20.0, true);
    render_text(surface, surface_w, surface_h, px + 24, card_y + 36, "Trackers & Ads Blocked", Color::TEXT_MUTED, 11.0, false);

    // Stat 2: Bandwidth saved
    let bw_str = shields.stats.bandwidth_saved_string();
    render_text(surface, surface_w, surface_h, px + 180, card_y + 12, &bw_str, Color::rgb(74, 222, 128), 20.0, true);
    render_text(surface, surface_w, surface_h, px + 180, card_y + 36, "Bandwidth Saved", Color::TEXT_MUTED, 11.0, false);

    // Protection Options List
    let opt_y = py + 148;
    render_option_row(surface, surface_w, surface_h, px + 16, opt_y, "Cross-site trackers blocked", is_up && shields.config.block_trackers_and_ads);
    render_option_row(surface, surface_w, surface_h, px + 16, opt_y + 26, "Connections upgraded to HTTPS", is_up && shields.config.upgrade_https);
    render_option_row(surface, surface_w, surface_h, px + 16, opt_y + 52, "Block all scripts (Strict)", is_up && shields.config.block_scripts);
    render_option_row(surface, surface_w, surface_h, px + 16, opt_y + 78, "Fingerprinting blocked", is_up && shields.config.prevent_fingerprinting);

    // Bottom note
    let foot_y = py + panel_h - 24;
    render_text(surface, surface_w, surface_h, px + 16, foot_y, "✓ Shield protection active on NovaOS", Color::rgb(148, 163, 184), 11.0, false);
}

fn render_option_row(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, label: &str, checked: bool) {
    let check_icon = if checked { "☑" } else { "☐" };
    let check_col = if checked { Color::rgb(56, 189, 248) } else { Color::rgb(100, 116, 139) };
    render_text(surface, sw, sh, x, y, check_icon, check_col, 13.0, true);
    render_text(surface, sw, sh, x + 20, y, label, Color::rgb(226, 232, 240), 12.0, false);
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

fn fill_rounded_rect(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32, h: i32, r: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, w, h, color);
}

fn draw_border(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32, h: i32, b: i32, _r: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, w, b, color);
    fill_rect(surface, sw, sh, x, y + h - b, w, b, color);
    fill_rect(surface, sw, sh, x, y, b, h, color);
    fill_rect(surface, sw, sh, x + w - b, y, b, h, color);
}
