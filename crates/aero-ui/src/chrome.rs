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

pub const TAB_STRIP_HEIGHT: i32 = 32;
pub const NAVBAR_HEIGHT: i32 = 38;
pub const BOOKMARKS_HEIGHT: i32 = 26;
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

    // 1. Tab Strip (y: 0..32)
    fill_rect(surface, surface_w, surface_h, 0, 0, w, TAB_STRIP_HEIGHT, Color::rgb(15, 23, 42));

    let tab_w = 160.min((w - 60) / tab_mgr.tabs.len().max(1) as i32);
    let mut tab_x = 8;

    for (idx, tab) in tab_mgr.tabs.iter().enumerate() {
        let is_active = idx == tab_mgr.active_index;
        let tab_bg = if is_active { Color::rgb(30, 41, 59) } else { Color::rgb(20, 27, 45) };

        fill_rounded_rect(surface, surface_w, surface_h, tab_x, 4, tab_w - 4, TAB_STRIP_HEIGHT - 4, 6, tab_bg);

        if is_active {
            // Brave Orange active tab indicator line
            fill_rect(surface, surface_w, surface_h, tab_x + 2, 4, tab_w - 8, 2, Color::BRAVE_ORANGE);
        }

        // Tab Title truncated
        let title_chars: String = tab.title.chars().take(14).collect();
        let text_col = if is_active { Color::WHITE } else { Color::TEXT_MUTED };
        render_text(surface, surface_w, surface_h, tab_x + 10, 12, &title_chars, text_col, 12.0, is_active);

        // Close Button (×)
        let close_x = tab_x + tab_w - 20;
        render_text(surface, surface_w, surface_h, close_x, 11, "×", Color::rgb(148, 163, 184), 14.0, false);

        tab_x += tab_w;
    }

    // New Tab (+) Button
    fill_rounded_rect(surface, surface_w, surface_h, tab_x + 4, 6, 24, 22, 4, Color::rgb(30, 41, 59));
    render_text(surface, surface_w, surface_h, tab_x + 11, 10, "+", Color::WHITE, 14.0, true);

    // 2. Navigation Bar (y: 32..70)
    let nav_y = TAB_STRIP_HEIGHT;
    fill_rect(surface, surface_w, surface_h, 0, nav_y, w, NAVBAR_HEIGHT, Color::rgb(30, 41, 59));
    fill_rect(surface, surface_w, surface_h, 0, nav_y + NAVBAR_HEIGHT - 1, w, 1, Color::rgb(51, 65, 85));

    // Nav Buttons: Back (←), Forward (→), Reload (↻), Home (⌂)
    let active_tab = tab_mgr.active_tab();
    let back_col = if active_tab.can_go_back() { Color::WHITE } else { Color::rgb(71, 85, 105) };
    let fwd_col = if active_tab.can_go_forward() { Color::WHITE } else { Color::rgb(71, 85, 105) };

    render_text(surface, surface_w, surface_h, 12, nav_y + 11, "←", back_col, 16.0, true);
    render_text(surface, surface_w, surface_h, 36, nav_y + 11, "→", fwd_col, 16.0, true);
    render_text(surface, surface_w, surface_h, 60, nav_y + 11, "↻", Color::WHITE, 16.0, true);
    render_text(surface, surface_w, surface_h, 84, nav_y + 11, "⌂", Color::WHITE, 16.0, true);

    // Omnibox URL Bar
    let omni_x = 112;
    let right_actions_w = 210;
    let omni_w = (w - omni_x - right_actions_w).max(200);
    let omni_h = 28;
    let omni_y = nav_y + 5;

    let omni_bg = if omnibox_focused { Color::rgb(15, 23, 42) } else { Color::rgb(20, 27, 45) };
    let omni_border = if omnibox_focused { Color::CHROME_BLUE } else { Color::rgb(51, 65, 85) };

    fill_rounded_rect(surface, surface_w, surface_h, omni_x, omni_y, omni_w, omni_h, 14, omni_bg);
    draw_border(surface, surface_w, surface_h, omni_x, omni_y, omni_w, omni_h, 1, 14, omni_border);

    // Security Badge inside Omnibox: Lock / Shield icon
    let is_secure = active_tab.url.starts_with("https://") || active_tab.url.starts_with("about:");
    let lock_col = if is_secure { Color::rgb(74, 222, 128) } else { Color::rgb(251, 146, 60) };
    let lock_icon = if is_secure { "🔒" } else { "!" };
    render_text(surface, surface_w, surface_h, omni_x + 10, omni_y + 7, lock_icon, lock_col, 12.0, true);

    // Omnibox Display Text
    let url_display = if omnibox_focused {
        omnibox_text
    } else {
        &active_tab.url
    };
    render_text(surface, surface_w, surface_h, omni_x + 30, omni_y + 7, url_display, Color::TEXT_MAIN, 13.0, false);

    if omnibox_focused {
        // Blinking cursor simulation
        let cursor_x = omni_x + 30 + (url_display.len() * 8) as i32;
        fill_rect(surface, surface_w, surface_h, cursor_x, omni_y + 6, 2, 16, Color::CHROME_BLUE);
    }

    // Bookmark star icon inside Omnibox right edge
    let is_starred = bookmarks.is_bookmarked(&active_tab.url);
    let star_col = if is_starred { Color::rgb(234, 179, 8) } else { Color::rgb(148, 163, 184) };
    render_text(surface, surface_w, surface_h, omni_x + omni_w - 24, omni_y + 7, "★", star_col, 14.0, is_starred);

    // Right Action Buttons:
    let mut action_x = omni_x + omni_w + 12;

    // 1. Brave Shields Badge Button `[🛡 X]`
    let shield_bg = if shields_open { Color::BRAVE_ORANGE } else { Color::rgb(40, 25, 20) };
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 56, omni_h, 14, shield_bg);
    draw_border(surface, surface_w, surface_h, action_x, omni_y, 56, omni_h, 1, 14, Color::BRAVE_ORANGE);
    let shield_badge = format!("🛡 {}", trackers_blocked.min(99));
    render_text(surface, surface_w, surface_h, action_x + 8, omni_y + 7, &shield_badge, Color::WHITE, 12.0, true);
    action_x += 66;

    // 2. Firefox Reader Mode Button `[📖]`
    let reader_bg = if active_tab.reader_mode { Color::rgb(59, 130, 246) } else { Color::rgb(20, 27, 45) };
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 6, reader_bg);
    render_text(surface, surface_w, surface_h, action_x + 10, omni_y + 7, "📖", Color::WHITE, 13.0, true);
    action_x += 44;

    // 3. Chrome DevTools Inspector Button `[⚙]`
    fill_rounded_rect(surface, surface_w, surface_h, action_x, omni_y, 36, omni_h, 6, Color::rgb(20, 27, 45));
    render_text(surface, surface_w, surface_h, action_x + 11, omni_y + 7, "⚙", Color::WHITE, 14.0, true);

    // 3. Bookmarks Bar (y: 70..96)
    let bm_y = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT;
    fill_rect(surface, surface_w, surface_h, 0, bm_y, w, BOOKMARKS_HEIGHT, Color::rgb(20, 27, 45));
    fill_rect(surface, surface_w, surface_h, 0, bm_y + BOOKMARKS_HEIGHT - 1, w, 1, Color::rgb(51, 65, 85));

    let mut bm_x = 12;
    for bm in &bookmarks.items {
        render_text(surface, surface_w, surface_h, bm_x, bm_y + 6, bm.icon, Color::rgb(251, 146, 60), 12.0, false);
        render_text(surface, surface_w, surface_h, bm_x + 14, bm_y + 6, &bm.title, Color::rgb(226, 232, 240), 11.0, false);
        bm_x += (bm.title.len() * 6) as i32 + 32;
    }
}

/// Hit-tests a mouse click against the browser chrome components.
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
        let tab_w = 160.min((w - 60) / tab_mgr.tabs.len().max(1) as i32);
        let mut tab_x = 8;

        for (idx, _) in tab_mgr.tabs.iter().enumerate() {
            if x >= tab_x && x < tab_x + tab_w {
                // Check if close button (×) clicked
                if x >= tab_x + tab_w - 24 {
                    return ChromeClickTarget::TabClose(idx);
                } else {
                    return ChromeClickTarget::TabSelect(idx);
                }
            }
            tab_x += tab_w;
        }

        // New Tab (+) Button
        if x >= tab_x && x < tab_x + 36 {
            return ChromeClickTarget::NewTab;
        }
    }

    // 2. Navigation Bar
    let nav_y = TAB_STRIP_HEIGHT;
    if y >= nav_y && y < nav_y + NAVBAR_HEIGHT {
        if x >= 8 && x < 30 {
            return ChromeClickTarget::NavBack;
        } else if x >= 32 && x < 54 {
            return ChromeClickTarget::NavForward;
        } else if x >= 56 && x < 78 {
            return ChromeClickTarget::NavReload;
        } else if x >= 80 && x < 104 {
            return ChromeClickTarget::NavHome;
        }

        let omni_x = 112;
        let right_actions_w = 210;
        let omni_w = (w - omni_x - right_actions_w).max(200);

        if x >= omni_x && x < omni_x + omni_w {
            return ChromeClickTarget::OmniboxFocus;
        }

        let mut action_x = omni_x + omni_w + 12;
        if x >= action_x && x < action_x + 56 {
            return ChromeClickTarget::ToggleShields;
        }
        action_x += 66;

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
            let item_w = (bm.title.len() * 6) as i32 + 32;
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
