//! # Aero Browser Chrome (Tab Strip, Navigation & Omnibox)
//!
//! Synthesizes the best interface patterns from Brave, Chrome, and Firefox:
//! - Sleek Chrome-style rounded tab strip with close buttons and `+` action
//! - Omnibox address bar with live security & TLS lock indicators
//! - Brave-style interactive Lion Shield button with blocked counter pill
//! - Firefox-style distraction-free Reader View trigger
//! - Quick Bookmarks bar and Chrome Developer Tools inspector toggle

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
use crate::bookmarks::BookmarkManager;
use crate::tab::{Tab, TabManager};

pub const TAB_STRIP_HEIGHT: i32 = 34;
pub const NAVBAR_HEIGHT: i32 = 40;
pub const BOOKMARKS_HEIGHT: i32 = 28;
pub const TOTAL_CHROME_HEIGHT: i32 = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT + BOOKMARKS_HEIGHT;

/// Chrome hit-test click targets for user interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChromeClickTarget {
    TabSelect(usize),
    TabClose(usize),
    NewTab,
    NavBack,
    NavForward,
    NavReload,
    NavHome,
    OmniboxFocus,
    ToggleShields,
    ToggleReader,
    ToggleDevTools,
    BookmarkClick(String),
    None,
}

/// Renders the complete browser chrome banner (Tabs, Navigation, Omnibox, Bookmarks).
pub fn render_chrome(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    tab_mgr: &TabManager,
    bookmarks: &BookmarkManager,
    shields_open: bool,
    trackers_blocked: u64,
    omnibox_text: &str,
    omnibox_focused: bool,
) {
    let w = surface_w as i32;

    // 1. Tab Strip (y: 0..34)
    fill_rect(surface, surface_w, surface_h, 0, 0, w, TAB_STRIP_HEIGHT, Color::rgb(15, 23, 42));

    let available_tab_w = w - 80;
    let tab_count = tab_mgr.tabs.len().max(1) as i32;
    let tab_w = (available_tab_w / tab_count).clamp(120, 200);
    let mut tab_x = 8;

    for (idx, tab) in tab_mgr.tabs.iter().enumerate() {
        let is_active = idx == tab_mgr.active_index;
        let tab_bg = if is_active { Color::rgb(30, 41, 59) } else { Color::rgb(20, 27, 45) };

        fill_rounded_rect(surface, surface_w, surface_h, tab_x, 4, tab_w - 6, TAB_STRIP_HEIGHT - 4, 6, tab_bg);

        if is_active {
            // Brave Orange active tab indicator line
            fill_rect(surface, surface_w, surface_h, tab_x + 2, 4, tab_w - 10, 2, Color::BRAVE_ORANGE);
        }

        // Tab Title truncated
        let max_title_chars = ((tab_w - 40) / 8).max(4) as usize;
        let title_chars: String = tab.title.chars().take(max_title_chars).collect();
        let text_col = if is_active { Color::WHITE } else { Color::TEXT_MUTED };
        render_text(surface, surface_w, surface_h, tab_x + 12, 13, &title_chars, text_col, 13.0, is_active);

        // Close Button (x)
        let close_cx = tab_x + tab_w - 18;
        let close_cy = 17;
        let close_col = if is_active { Color::rgb(148, 163, 184) } else { Color::rgb(100, 116, 139) };
        draw_close_x(surface, surface_w, surface_h, close_cx, close_cy, close_col);

        tab_x += tab_w;
    }

    // New Tab (+) Button
    let new_tab_x = tab_x + 4;
    fill_rounded_rect(surface, surface_w, surface_h, new_tab_x, 6, 26, 24, 4, Color::rgb(30, 41, 59));
    render_text(surface, surface_w, surface_h, new_tab_x + 9, 10, "+", Color::WHITE, 14.0, true);

    // 2. Navigation Bar (y: 34..74)
    let nav_y = TAB_STRIP_HEIGHT;
    fill_rect(surface, surface_w, surface_h, 0, nav_y, w, NAVBAR_HEIGHT, Color::rgb(30, 41, 59));
    fill_rect(surface, surface_w, surface_h, 0, nav_y + NAVBAR_HEIGHT - 1, w, 1, Color::rgb(51, 65, 85));

    // Nav Buttons: Back, Forward, Reload, Home
    let active_tab = tab_mgr.active_tab();
    let back_col = if active_tab.can_go_back() { Color::WHITE } else { Color::rgb(71, 85, 105) };
    let fwd_col = if active_tab.can_go_forward() { Color::WHITE } else { Color::rgb(71, 85, 105) };

    draw_back_arrow(surface, surface_w, surface_h, 16, nav_y + 20, back_col);
    draw_forward_arrow(surface, surface_w, surface_h, 40, nav_y + 20, fwd_col);
    draw_reload_icon(surface, surface_w, surface_h, 64, nav_y + 20, Color::WHITE);
    draw_home_icon(surface, surface_w, surface_h, 88, nav_y + 20, Color::WHITE);

    // Omnibox URL Bar
    let omni_x = 112;
    let right_actions_w = 230;
    let omni_w = (w - omni_x - right_actions_w).max(220);
    let omni_h = 30;
    let omni_y = nav_y + 5;

    let omni_bg = if omnibox_focused { Color::rgb(15, 23, 42) } else { Color::rgb(20, 27, 45) };
    let omni_border = if omnibox_focused { Color::CHROME_BLUE } else { Color::rgb(51, 65, 85) };

    fill_rounded_rect(surface, surface_w, surface_h, omni_x, omni_y, omni_w, omni_h, 14, omni_bg);
    draw_border(surface, surface_w, surface_h, omni_x, omni_y, omni_w, omni_h, 1, 14, omni_border);

    // Security Badge inside Omnibox: Green Lock or Shield
    let is_secure = active_tab.url.starts_with("https://") || active_tab.url.starts_with("about:");
    let lock_col = if is_secure { Color::rgb(74, 222, 128) } else { Color::rgb(251, 146, 60) };
    draw_lock_icon(surface, surface_w, surface_h, omni_x + 10, omni_y + 8, lock_col);

    // Omnibox Display Text
    let url_display = if omnibox_focused {
        omnibox_text
    } else {
        &active_tab.url
    };
    render_text(surface, surface_w, surface_h, omni_x + 30, omni_y + 8, url_display, Color::TEXT_MAIN, 13.0, false);

    if omnibox_focused {
        // Blinking cursor simulation
        let cursor_x = omni_x + 30 + (url_display.len() * 8) as i32;
        fill_rect(surface, surface_w, surface_h, cursor_x, omni_y + 7, 2, 16, Color::CHROME_BLUE);
    }

    // Bookmark star icon inside Omnibox right edge
    let is_starred = bookmarks.is_bookmarked(&active_tab.url);
    let star_col = if is_starred { Color::rgb(234, 179, 8) } else { Color::rgb(148, 163, 184) };
    draw_star_icon(surface, surface_w, surface_h, omni_x + omni_w - 20, omni_y + 15, star_col);

    // Right Action Buttons:
    let mut action_x = omni_x + omni_w + 12;

    // 1. Brave Shields Badge Button `[Shield X]`
    let shield_bg = if shields_open { Color::BRAVE_ORANGE } else { Color::rgb(40, 25, 20) };
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 64, omni_h, 14, shield_bg);
    draw_border(surface, surface_w, surface_h, action_x, omni_y, 64, omni_h, 1, 14, Color::BRAVE_ORANGE);
    draw_shield_icon(surface, surface_w, surface_h, action_x + 8, omni_y + 8, Color::WHITE);
    let shield_badge = format!("{}", trackers_blocked.min(99));
    render_text(surface, surface_w, surface_h, action_x + 28, omni_y + 9, &shield_badge, Color::WHITE, 12.0, true);
    action_x += 74;

    // 2. Firefox Reader View Button
    let reader_bg = if active_tab.reader_mode { Color::FIREFOX_ACCENT } else { Color::rgb(20, 27, 45) };
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 6, reader_bg);
    draw_border(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 1, 6, Color::rgb(51, 65, 85));
    draw_reader_icon(surface, surface_w, surface_h, action_x + 11, omni_y + 9, Color::WHITE);
    action_x += 44;

    // 3. Chrome DevTools Toggle Button `</>`
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 6, Color::rgb(20, 27, 45));
    draw_border(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 1, 6, Color::rgb(51, 65, 85));
    draw_devtools_icon(surface, surface_w, surface_h, action_x + 10, omni_y + 10, Color::rgb(56, 189, 248));

    // 3. Bookmarks Bar (y: 74..102)
    let bm_y = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT;
    fill_rect(surface, surface_w, surface_h, 0, bm_y, w, BOOKMARKS_HEIGHT, Color::rgb(20, 27, 45));
    fill_rect(surface, surface_w, surface_h, 0, bm_y + BOOKMARKS_HEIGHT - 1, w, 1, Color::rgb(30, 41, 59));

    let mut bm_x = 12;
    for bm in &bookmarks.items {
        draw_star_icon(surface, surface_w, surface_h, bm_x + 5, bm_y + 14, Color::rgb(234, 179, 8));
        render_text(surface, surface_w, surface_h, bm_x + 16, bm_y + 7, &bm.title, Color::rgb(226, 232, 240), 12.0, false);
        let item_w = (bm.title.len() * 8) as i32 + 28;
        bm_x += item_w;
    }
}

/// Hit-tests an (x, y) click against the chrome components.
pub fn hit_test_chrome(
    x: i32,
    y: i32,
    surface_w: usize,
    tab_mgr: &TabManager,
    bookmarks: &BookmarkManager,
) -> ChromeClickTarget {
    let w = surface_w as i32;

    // 1. Tab Strip
    if y >= 0 && y < TAB_STRIP_HEIGHT {
        let available_tab_w = w - 80;
        let tab_count = tab_mgr.tabs.len().max(1) as i32;
        let tab_w = (available_tab_w / tab_count).clamp(120, 200);
        let mut tab_x = 8;

        for idx in 0..tab_mgr.tabs.len() {
            if x >= tab_x && x < tab_x + tab_w - 6 {
                let close_cx = tab_x + tab_w - 18;
                if x >= close_cx - 8 && x <= close_cx + 8 && y >= 8 && y <= 26 {
                    return ChromeClickTarget::TabClose(idx);
                }
                return ChromeClickTarget::TabSelect(idx);
            }
            tab_x += tab_w;
        }

        if x >= tab_x + 4 && x < tab_x + 30 {
            return ChromeClickTarget::NewTab;
        }
    }

    // 2. Navigation Bar
    let nav_y = TAB_STRIP_HEIGHT;
    if y >= nav_y && y < nav_y + NAVBAR_HEIGHT {
        if x >= 8 && x < 28 {
            return ChromeClickTarget::NavBack;
        } else if x >= 30 && x < 50 {
            return ChromeClickTarget::NavForward;
        } else if x >= 52 && x < 74 {
            return ChromeClickTarget::NavReload;
        } else if x >= 76 && x < 98 {
            return ChromeClickTarget::NavHome;
        }

        let omni_x = 112;
        let right_actions_w = 230;
        let omni_w = (w - omni_x - right_actions_w).max(220);

        if x >= omni_x && x < omni_x + omni_w {
            return ChromeClickTarget::OmniboxFocus;
        }

        let mut action_x = omni_x + omni_w + 12;
        if x >= action_x && x < action_x + 64 {
            return ChromeClickTarget::ToggleShields;
        }
        action_x += 74;

        if x >= action_x && x < action_x + 36 {
            return ChromeClickTarget::ToggleReader;
        }
        action_x += 44;

        if x >= action_x && x < action_x + 36 {
            return ChromeClickTarget::ToggleDevTools;
        }
    }

    // 3. Bookmarks Bar
    let bm_y = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT;
    if y >= bm_y && y < bm_y + BOOKMARKS_HEIGHT {
        let mut bm_x = 12;
        for bm in &bookmarks.items {
            let item_w = (bm.title.len() * 8) as i32 + 28;
            if x >= bm_x && x < bm_x + item_w {
                return ChromeClickTarget::BookmarkClick(bm.url.clone());
            }
            bm_x += item_w;
        }
    }

    ChromeClickTarget::None
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

fn fill_rounded_rect(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32, h: i32, _r: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, w, h, color);
}

fn draw_border(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32, h: i32, b: i32, _r: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, w, b, color);
    fill_rect(surface, sw, sh, x, y + h - b, w, b, color);
    fill_rect(surface, sw, sh, x, y, b, h, color);
    fill_rect(surface, sw, sh, x + w - b, y, b, h, color);
}

fn draw_back_arrow(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    fill_rect(surface, sw, sh, cx - 4, cy, 9, 2, color);
    for d in 0..5 {
        fill_rect(surface, sw, sh, cx - 4 + d, cy - d, 2, 2, color);
        fill_rect(surface, sw, sh, cx - 4 + d, cy + d, 2, 2, color);
    }
}

fn draw_forward_arrow(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    fill_rect(surface, sw, sh, cx - 4, cy, 9, 2, color);
    for d in 0..5 {
        fill_rect(surface, sw, sh, cx + 4 - d, cy - d, 2, 2, color);
        fill_rect(surface, sw, sh, cx + 4 - d, cy + d, 2, 2, color);
    }
}

fn draw_reload_icon(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    // 3/4 circle arc (diameter 10)
    for dy in -4..=4 {
        for dx in -4..=4 {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= 9 && dist_sq <= 18 && !(dx > 1 && dy < -1) {
                fill_rect(surface, sw, sh, cx + dx, cy + dy, 1, 1, color);
            }
        }
    }
    // Arrow head at top-right
    fill_rect(surface, sw, sh, cx + 1, cy - 6, 4, 2, color);
    fill_rect(surface, sw, sh, cx + 3, cy - 6, 2, 4, color);
}

fn draw_home_icon(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    for dy in 0..5 {
        let hw = dy;
        fill_rect(surface, sw, sh, cx - hw, cy - 5 + dy, hw * 2 + 1, 1, color);
    }
    fill_rect(surface, sw, sh, cx - 4, cy, 9, 6, color);
    fill_rect(surface, sw, sh, cx - 1, cy + 2, 3, 4, Color::rgb(30, 41, 59));
}

fn draw_lock_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x + 2, y, 6, 2, color);
    fill_rect(surface, sw, sh, x + 1, y + 1, 2, 5, color);
    fill_rect(surface, sw, sh, x + 7, y + 1, 2, 5, color);
    fill_rect(surface, sw, sh, x, y + 6, 10, 8, color);
    fill_rect(surface, sw, sh, x + 4, y + 9, 2, 3, Color::rgb(15, 23, 42));
}

fn draw_shield_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, 12, 6, color);
    for dy in 0..6 {
        let w = (12 - dy * 2).max(2);
        fill_rect(surface, sw, sh, x + dy, y + 6 + dy, w, 1, color);
    }
    fill_rect(surface, sw, sh, x + 5, y + 2, 2, 6, Color::rgb(255, 85, 0));
    fill_rect(surface, sw, sh, x + 3, y + 4, 6, 2, Color::rgb(255, 85, 0));
}

fn draw_star_icon(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    fill_rect(surface, sw, sh, cx, cy - 4, 1, 9, color);
    fill_rect(surface, sw, sh, cx - 4, cy, 9, 1, color);
    fill_rect(surface, sw, sh, cx - 2, cy - 2, 5, 5, color);
    fill_rect(surface, sw, sh, cx - 1, cy - 3, 3, 7, color);
    fill_rect(surface, sw, sh, cx - 3, cy - 1, 7, 3, color);
}

fn draw_reader_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, 6, 10, color);
    fill_rect(surface, sw, sh, x + 7, y, 6, 10, color);
    fill_rect(surface, sw, sh, x + 6, y, 1, 10, Color::rgb(30, 41, 59));
    fill_rect(surface, sw, sh, x + 1, y + 2, 4, 1, Color::rgb(30, 41, 59));
    fill_rect(surface, sw, sh, x + 1, y + 5, 4, 1, Color::rgb(30, 41, 59));
    fill_rect(surface, sw, sh, x + 8, y + 2, 4, 1, Color::rgb(30, 41, 59));
    fill_rect(surface, sw, sh, x + 8, y + 5, 4, 1, Color::rgb(30, 41, 59));
}

fn draw_devtools_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y + 3, 2, 2, color);
    fill_rect(surface, sw, sh, x + 2, y + 1, 2, 2, color);
    fill_rect(surface, sw, sh, x + 2, y + 5, 2, 2, color);

    fill_rect(surface, sw, sh, x + 6, y, 2, 2, color);
    fill_rect(surface, sw, sh, x + 5, y + 3, 2, 2, color);
    fill_rect(surface, sw, sh, x + 4, y + 6, 2, 2, color);

    fill_rect(surface, sw, sh, x + 9, y + 3, 2, 2, color);
    fill_rect(surface, sw, sh, x + 7, y + 1, 2, 2, color);
    fill_rect(surface, sw, sh, x + 7, y + 5, 2, 2, color);
}

fn draw_close_x(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    for d in 0..7 {
        fill_rect(surface, sw, sh, cx - 3 + d, cy - 3 + d, 2, 1, color);
        fill_rect(surface, sw, sh, cx - 3 + d, cy + 3 - d, 2, 1, color);
    }
}
