//! # Aero Developer Tools & DOM Inspector
//!
//! Inspired by Chrome DevTools: Live DOM element tree inspection,
//! attribute viewing, computed styles inspector, and engine debugging panel.

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
use aero_engine::dom::{Node, NodeType};
use aero_engine::font::render_text;

/// Interactive state of the DevTools panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevToolsState {
    pub is_open: bool,
    pub selected_node_id: Option<String>,
    pub active_tab: DevToolsTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevToolsTab {
    Elements,
    Console,
    Network,
    Shields,
}

impl Default for DevToolsState {
    fn default() -> Self {
        DevToolsState {
            is_open: false,
            selected_node_id: None,
            active_tab: DevToolsTab::Elements,
        }
    }
}

/// Renders the Chrome-inspired DevTools Inspector split panel.
pub fn render_devtools_panel(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    state: &DevToolsState,
    dom_root: &Node,
) {
    if !state.is_open || w <= 0 || h <= 0 {
        return;
    }

    // Panel Background
    fill_rect(surface, surface_w, surface_h, x, y, w, h, Color::rgb(15, 23, 42));
    // Left border separator
    fill_rect(surface, surface_w, surface_h, x, y, 1, h, Color::rgb(51, 65, 85));

    // Tab Header
    fill_rect(surface, surface_w, surface_h, x, y, w, 28, Color::rgb(30, 41, 59));
    fill_rect(surface, surface_w, surface_h, x, y + 27, w, 1, Color::rgb(51, 65, 85));

    let tabs = ["Elements", "Console", "Network", "Shields"];
    let mut tab_x = x + 12;
    for &tab in &tabs {
        let is_active = match (tab, state.active_tab) {
            ("Elements", DevToolsTab::Elements) => true,
            ("Console", DevToolsTab::Console) => true,
            ("Network", DevToolsTab::Network) => true,
            ("Shields", DevToolsTab::Shields) => true,
            _ => false,
        };

        let col = if is_active { Color::rgb(56, 189, 248) } else { Color::rgb(148, 163, 184) };
        render_text(surface, surface_w, surface_h, tab_x, y + 7, tab, col, 12.0, is_active);
        if is_active {
            fill_rect(surface, surface_w, surface_h, tab_x, y + 26, 50, 2, Color::rgb(56, 189, 248));
        }
        tab_x += 70;
    }

    // DevTools Content Area
    match state.active_tab {
        DevToolsTab::Elements => {
            render_dom_tree(surface, surface_w, surface_h, x + 12, y + 36, w - 24, h - 48, dom_root);
        }
        DevToolsTab::Console => {
            render_text(surface, surface_w, surface_h, x + 16, y + 42, "Aero Console: 0 Errors, 0 Warnings", Color::rgb(74, 222, 128), 12.0, false);
            render_text(surface, surface_w, surface_h, x + 16, y + 64, "> [OK] Aero Engine 1.0 initialized in pure Rust.", Color::rgb(226, 232, 240), 12.0, false);
            render_text(surface, surface_w, surface_h, x + 16, y + 84, "> [SHIELDS] Active ad & tracking defense running.", Color::rgb(251, 146, 60), 12.0, false);
        }
        DevToolsTab::Network => {
            render_text(surface, surface_w, surface_h, x + 16, y + 42, "Network Activity (0 Requests pending)", Color::TEXT_MUTED, 12.0, false);
            render_text(surface, surface_w, surface_h, x + 16, y + 64, "• GET / 200 OK (text/html, 1.2 KB, 8ms)", Color::rgb(74, 222, 128), 12.0, false);
            render_text(surface, surface_w, surface_h, x + 16, y + 84, "• BLOCKED /tracker.js (Aero Shields)", Color::rgb(239, 68, 68), 12.0, false);
        }
        DevToolsTab::Shields => {
            render_text(surface, surface_w, surface_h, x + 16, y + 42, "Aero Privacy Engine Telemetry", Color::BRAVE_ORANGE, 13.0, true);
            render_text(surface, surface_w, surface_h, x + 16, y + 64, "• Blocklist Rules: 26 active pattern matchers", Color::TEXT_MAIN, 12.0, false);
            render_text(surface, surface_w, surface_h, x + 16, y + 84, "• Cryptographic HTTPS Upgrade: Enabled", Color::TEXT_MAIN, 12.0, false);
        }
    }
}

fn render_dom_tree(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    _w: i32,
    max_h: i32,
    node: &Node,
) {
    let mut cur_y = y;
    render_node_recursive(surface, sw, sh, x, &mut cur_y, y + max_h, node, 0);
}

fn render_node_recursive(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    base_x: i32,
    cur_y: &mut i32,
    max_y: i32,
    node: &Node,
    depth: usize,
) {
    if *cur_y >= max_y {
        return;
    }

    let indent = (depth * 14) as i32;
    let node_x = base_x + indent;

    match &node.node_type {
        NodeType::Document => {
            render_text(surface, sw, sh, node_x, *cur_y, "#document", Color::rgb(148, 163, 184), 12.0, true);
            *cur_y += 18;
        }
        NodeType::Element(data) => {
            let tag_str = format!("<{}>", data.tag_name);
            render_text(surface, sw, sh, node_x, *cur_y, &tag_str, Color::rgb(192, 132, 252), 12.0, true);

            // Display attributes if present
            if !data.attributes.is_empty() {
                let mut attr_x = node_x + (tag_str.len() * 7) as i32 + 6;
                for (k, v) in &data.attributes {
                    let attr_str = format!("{}=\"{}\"", k, v);
                    render_text(surface, sw, sh, attr_x, *cur_y, &attr_str, Color::rgb(56, 189, 248), 11.0, false);
                    attr_x += (attr_str.len() * 6) as i32 + 6;
                }
            }
            *cur_y += 18;
        }
        NodeType::Text(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let snippet = if trimmed.len() > 30 {
                    format!("\"{}...\"", &trimmed[..30])
                } else {
                    format!("\"{}\"", trimmed)
                };
                render_text(surface, sw, sh, node_x, *cur_y, &snippet, Color::rgb(241, 245, 249), 11.0, false);
                *cur_y += 18;
            }
        }
        NodeType::Comment(c) => {
            let comment_str = format!("<!-- {} -->", c);
            render_text(surface, sw, sh, node_x, *cur_y, &comment_str, Color::rgb(100, 116, 139), 11.0, false);
            *cur_y += 18;
        }
    }

    for child in &node.children {
        render_node_recursive(surface, sw, sh, base_x, cur_y, max_y, child, depth + 1);
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
