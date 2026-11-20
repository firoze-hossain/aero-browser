//! # Aero Browser Chrome (Tab Strip, Navigation & Omnibox)
//!
//! Synthesizes the best interface patterns from Brave, Chrome, and Firefox:
//! - Sleek Chrome-style rounded tab strip with close buttons and `+` action
//! - Omnibox address bar with live security & TLS lock indicators
//! - Brave-style interactive Lion Shield button with blocked counter pill
//! - Firefox-style distraction-free Reader View trigger
//! - Quick Bookmarks bar with interactive Folder dropdowns (Brave/Chrome style)
//! - Main hamburger toggle menu [≡] with full browser options (Image 3)
//! - Secondary "More tools >" flyout popover with Developer Tools (Image 5)

#![allow(unused)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};

use aero_engine::css::Color;
use aero_engine::font::render_text;
use crate::bookmarks::{BookmarkItem, BookmarkManager};
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
    ToggleMainMenu,
    BookmarkFolderClick(usize),
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
    main_menu_open: bool,
    open_bookmark_folder: Option<usize>,
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
    let right_actions_w = 270;
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
    action_x += 44;

    // 4. Hamburger Main Menu Toggle Button `[≡]` (Image 3 trigger)
    let menu_btn_x = w - 38;
    let menu_btn_bg = if main_menu_open { Color::rgb(51, 65, 85) } else { Color::rgb(20, 27, 45) };
    fill_rounded_rect(surface, surface_w, surface_h, menu_btn_x, omni_y, 30, omni_h, 6, menu_btn_bg);
    draw_border(surface, surface_w, surface_h, menu_btn_x, omni_y, 30, omni_h, 1, 6, Color::rgb(51, 65, 85));
    draw_hamburger_icon(surface, surface_w, surface_h, menu_btn_x + 8, omni_y + 8, Color::WHITE);

    // 3. Bookmarks Bar (y: 74..102) - Image 1 & 2
    let bm_y = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT;
    fill_rect(surface, surface_w, surface_h, 0, bm_y, w, BOOKMARKS_HEIGHT, Color::rgb(20, 27, 45));
    fill_rect(surface, surface_w, surface_h, 0, bm_y + BOOKMARKS_HEIGHT - 1, w, 1, Color::rgb(30, 41, 59));

    let all_bm_w = 110;
    let all_bm_x = w - all_bm_w - 8;

    let mut bm_x = 12;
    for (idx, bm) in bookmarks.items.iter().enumerate() {
        if idx == bookmarks.items.len() - 1 {
            // "All Bookmarks" rendered on the far right
            continue;
        }

        let is_folder = bm.is_folder();
        let is_open = open_bookmark_folder == Some(idx);
        let item_title = bm.title();
        let item_w = (item_title.len() * 8) as i32 + 28;

        if bm_x + item_w > all_bm_x - 8 {
            break;
        }

        if is_open {
            fill_rounded_rect(surface, surface_w, surface_h, bm_x, bm_y + 3, item_w - 4, BOOKMARKS_HEIGHT - 6, 4, Color::rgb(51, 65, 85));
        }

        if is_folder {
            draw_folder_icon(surface, surface_w, surface_h, bm_x + 5, bm_y + 9, Color::rgb(203, 213, 225));
        } else {
            draw_star_icon(surface, surface_w, surface_h, bm_x + 5, bm_y + 14, Color::rgb(234, 179, 8));
        }

        let text_col = if is_open { Color::WHITE } else { Color::rgb(226, 232, 240) };
        render_text(surface, surface_w, surface_h, bm_x + 20, bm_y + 7, item_title, text_col, 12.0, is_open);

        bm_x += item_w;
    }

    // Render "All Bookmarks" on far right of bookmarks bar
    let is_all_open = open_bookmark_folder == Some(bookmarks.items.len() - 1);
    if is_all_open {
        fill_rounded_rect(surface, surface_w, surface_h, all_bm_x, bm_y + 3, all_bm_w, BOOKMARKS_HEIGHT - 6, 4, Color::rgb(51, 65, 85));
    }
    draw_folder_icon(surface, surface_w, surface_h, all_bm_x + 6, bm_y + 9, Color::rgb(203, 213, 225));
    render_text(surface, surface_w, surface_h, all_bm_x + 24, bm_y + 7, "All Bookmarks", Color::rgb(226, 232, 240), 12.0, is_all_open);
}

/// Renders a bookmark folder dropdown popover (Image 2: 7th Dream Pr... dropdown).
pub fn render_bookmark_folder_dropdown(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    folder_x: i32,
    folder_item: &BookmarkItem,
) {
    let w = surface_w as i32;
    let children = match folder_item.children() {
        Some(c) => c,
        None => return,
    };

    let item_h = 30;
    let dd_w = 320;
    let dd_h = (children.len() as i32 * item_h) + 12;
    let dd_x = folder_x.min(w - dd_w - 8).max(8);
    let dd_y = TOTAL_CHROME_HEIGHT;

    // Drop shadow
    fill_rect(surface, surface_w, surface_h, dd_x + 3, dd_y + 3, dd_w, dd_h, Color::rgb(0, 0, 0));
    // Popover container
    fill_rounded_rect(surface, surface_w, surface_h, dd_x, dd_y, dd_w, dd_h, 6, Color::rgb(24, 32, 47));
    draw_border(surface, surface_w, surface_h, dd_x, dd_y, dd_w, dd_h, 1, 6, Color::rgb(51, 65, 85));

    let mut item_y = dd_y + 6;
    for child in children {
        let is_folder = child.is_folder();
        let title = child.title();

        if is_folder {
            draw_folder_icon(surface, surface_w, surface_h, dd_x + 12, item_y + 9, Color::rgb(203, 213, 225));
            draw_chevron_right(surface, surface_w, surface_h, dd_x + dd_w - 18, item_y + 11, Color::rgb(148, 163, 184));
        } else if title.contains("Google Sheets") || title.contains("xlsx") {
            draw_sheets_icon(surface, surface_w, surface_h, dd_x + 12, item_y + 9);
        } else if title.contains("drive.google.com") {
            draw_folder_icon(surface, surface_w, surface_h, dd_x + 12, item_y + 9, Color::rgb(56, 189, 248));
        } else {
            draw_star_icon(surface, surface_w, surface_h, dd_x + 18, item_y + 15, Color::rgb(234, 179, 8));
        }

        let max_chars = ((dd_w - 55) / 8).max(4) as usize;
        let display_title: String = title.chars().take(max_chars).collect();
        render_text(surface, surface_w, surface_h, dd_x + 32, item_y + 8, &display_title, Color::rgb(241, 245, 249), 12.0, false);

        item_y += item_h;
    }
}

/// Renders the complete Brave Main Toggle Menu (Image 3).
pub fn render_main_toggle_menu(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
    show_more_tools: bool,
) {
    let w = surface_w as i32;
    let menu_w = 350;
    let menu_h = 574;
    let menu_x = w - menu_w - 12;
    let menu_y = TOTAL_CHROME_HEIGHT - 4;

    // Drop shadow
    fill_rect(surface, surface_w, surface_h, menu_x + 4, menu_y + 4, menu_w, menu_h, Color::rgb(0, 0, 0));
    // Background & border
    fill_rounded_rect(surface, surface_w, surface_h, menu_x, menu_y, menu_w, menu_h, 8, Color::rgb(24, 32, 47));
    draw_border(surface, surface_w, surface_h, menu_x, menu_y, menu_w, menu_h, 1, 8, Color::rgb(51, 65, 85));

    let mut cur_y = menu_y + 8;
    let item_h = 24;

    // Rows from Image 3:
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "New tab", "Ctrl+T", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "New window", "Ctrl+N", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "New private window", "Ctrl+Shift+N", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "New private window with Tor", "Shift+Alt+N", false, false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Leo AI", "", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Wallet", "", false, false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    // Sidebar: On | Autohide | Off (with Off selected)
    render_text(surface, surface_w, surface_h, menu_x + 16, cur_y + 4, "Sidebar", Color::WHITE, 12.0, false);
    let pill_x = menu_x + menu_w - 128;
    fill_rounded_rect(surface, surface_w, surface_h, pill_x, cur_y - 1, 114, 20, 4, Color::rgb(15, 23, 42));
    render_text(surface, surface_w, surface_h, pill_x + 6, cur_y + 3, "On", Color::rgb(148, 163, 184), 10.0, false);
    render_text(surface, surface_w, surface_h, pill_x + 30, cur_y + 3, "Autohide", Color::rgb(148, 163, 184), 10.0, false);
    fill_rounded_rect(surface, surface_w, surface_h, pill_x + 82, cur_y, 30, 18, 3, Color::rgb(59, 130, 246));
    render_text(surface, surface_w, surface_h, pill_x + 88, cur_y + 3, "Off", Color::WHITE, 10.0, true);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Passwords and autofill", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "History", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Bookmarks and lists", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Downloads", "Ctrl+J", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Extensions", "", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Delete browsing data...", "Ctrl+Shift+Del", false, false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    // Zoom: - 100% + ⛶
    render_text(surface, surface_w, surface_h, menu_x + 16, cur_y + 4, "Zoom", Color::WHITE, 12.0, false);
    let zoom_x = menu_x + menu_w - 100;
    render_text(surface, surface_w, surface_h, zoom_x, cur_y + 3, "-", Color::WHITE, 13.0, true);
    render_text(surface, surface_w, surface_h, zoom_x + 18, cur_y + 4, "100%", Color::rgb(226, 232, 240), 11.0, false);
    render_text(surface, surface_w, surface_h, zoom_x + 58, cur_y + 3, "+", Color::WHITE, 13.0, true);
    render_text(surface, surface_w, surface_h, zoom_x + 78, cur_y + 3, "[]", Color::rgb(148, 163, 184), 11.0, false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Print...", "Ctrl+P", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Find and edit", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Save and share", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "More tools", "", true, show_more_tools);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, menu_x, cur_y, menu_w);
    cur_y += 7;

    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Help", "", true, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Settings", "", false, false);
    cur_y += item_h;
    draw_menu_row(surface, surface_w, surface_h, menu_x, cur_y, menu_w, "Exit", "", false, false);
}

fn draw_menu_row(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    label: &str,
    shortcut: &str,
    chevron: bool,
    is_active: bool,
) {
    if is_active {
        fill_rounded_rect(surface, sw, sh, x + 4, y - 2, w - 8, 24, 4, Color::rgb(51, 65, 85));
    }
    render_text(surface, sw, sh, x + 16, y + 4, label, Color::WHITE, 12.0, is_active);
    if !shortcut.is_empty() {
        let sc_x = x + w - (shortcut.len() * 8) as i32 - 16;
        render_text(surface, sw, sh, sc_x, y + 4, shortcut, Color::rgb(148, 163, 184), 11.0, false);
    }
    if chevron {
        draw_chevron_right(surface, sw, sh, x + w - 18, y + 7, Color::rgb(148, 163, 184));
    }
}

fn draw_menu_divider(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, w: i32) {
    fill_rect(surface, sw, sh, x + 8, y + 3, w - 16, 1, Color::rgb(45, 55, 75));
}

/// Renders the "More tools >" secondary flyout submenu (Image 5).
pub fn render_more_tools_submenu(
    surface: &mut [u32],
    surface_w: usize,
    surface_h: usize,
) {
    let w = surface_w as i32;
    let sub_w = 260;
    let sub_h = 330;
    let main_menu_w = 350;
    let menu_x = w - main_menu_w - 12;
    let sub_x = menu_x - sub_w - 4;
    let sub_y = TOTAL_CHROME_HEIGHT + 236;

    // Drop shadow
    fill_rect(surface, surface_w, surface_h, sub_x + 4, sub_y + 4, sub_w, sub_h, Color::rgb(0, 0, 0));
    // Container
    fill_rounded_rect(surface, surface_w, surface_h, sub_x, sub_y, sub_w, sub_h, 8, Color::rgb(24, 32, 47));
    draw_border(surface, surface_w, surface_h, sub_x, sub_y, sub_w, sub_h, 1, 8, Color::rgb(51, 65, 85));

    let mut cur_y = sub_y + 8;
    let item_h = 24;

    // Items from Image 5:
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Add new profile", "", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Open Guest profile", "", false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, sub_x, cur_y, sub_w);
    cur_y += 7;

    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Sync", "", false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, sub_x, cur_y, sub_w);
    cur_y += 7;

    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Tab search", "Ctrl+Shift+A", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Name window...", "", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Quick commands", "Ctrl+Space", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Show apps", "", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Show tabs vertically", "", false);
    cur_y += item_h;
    draw_menu_divider(surface, surface_w, surface_h, sub_x, cur_y, sub_w);
    cur_y += 7;

    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Performance", "", false);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Developer tools", "Ctrl+Shift+I", true);
    cur_y += item_h;
    draw_submenu_row(surface, surface_w, surface_h, sub_x, cur_y, sub_w, "Task manager", "Shift+Esc", false);
}

fn draw_submenu_row(
    surface: &mut [u32],
    sw: usize,
    sh: usize,
    x: i32,
    y: i32,
    w: i32,
    label: &str,
    shortcut: &str,
    is_highlighted: bool,
) {
    if is_highlighted {
        fill_rounded_rect(surface, sw, sh, x + 4, y - 2, w - 8, 24, 4, Color::rgb(51, 65, 85));
    }
    render_text(surface, sw, sh, x + 14, y + 4, label, Color::WHITE, 12.0, is_highlighted);
    if !shortcut.is_empty() {
        let sc_x = x + w - (shortcut.len() * 8) as i32 - 14;
        render_text(surface, sw, sh, sc_x, y + 4, shortcut, Color::rgb(148, 163, 184), 11.0, false);
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
        let right_actions_w = 270;
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

        // Hamburger Main Menu button at top right
        let menu_x = w - 42;
        if x >= menu_x && x <= w - 6 {
            return ChromeClickTarget::ToggleMainMenu;
        }
    }

    // 3. Bookmarks Bar
    let bm_y = TAB_STRIP_HEIGHT + NAVBAR_HEIGHT;
    if y >= bm_y && y < bm_y + BOOKMARKS_HEIGHT {
        let all_bm_w = 110;
        let all_bm_x = w - all_bm_w - 8;
        if x >= all_bm_x && x < all_bm_x + all_bm_w {
            return ChromeClickTarget::BookmarkFolderClick(bookmarks.items.len() - 1);
        }

        let mut bm_x = 12;
        for (idx, bm) in bookmarks.items.iter().enumerate() {
            if idx == bookmarks.items.len() - 1 {
                continue;
            }
            let item_w = (bm.title().len() * 8) as i32 + 28;
            if x >= bm_x && x < bm_x + item_w {
                if bm.is_folder() {
                    return ChromeClickTarget::BookmarkFolderClick(idx);
                } else if let Some(url) = bm.url() {
                    return ChromeClickTarget::BookmarkClick(url.to_owned());
                }
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
    for dy in -4..=4 {
        for dx in -4..=4 {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= 9 && dist_sq <= 18 && !(dx > 1 && dy < -1) {
                fill_rect(surface, sw, sh, cx + dx, cy + dy, 1, 1, color);
            }
        }
    }
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

fn draw_hamburger_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y + 2, 14, 2, color);
    fill_rect(surface, sw, sh, x, y + 6, 14, 2, color);
    fill_rect(surface, sw, sh, x, y + 10, 14, 2, color);
}

fn draw_folder_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    fill_rect(surface, sw, sh, x, y, 6, 2, color);
    fill_rect(surface, sw, sh, x, y + 2, 13, 8, color);
    fill_rect(surface, sw, sh, x + 1, y + 3, 11, 2, Color::rgb(30, 41, 59));
}

fn draw_chevron_right(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32, color: Color) {
    for d in 0..4 {
        fill_rect(surface, sw, sh, x + d, y + d, 2, 1, color);
        fill_rect(surface, sw, sh, x + d, y + 6 - d, 2, 1, color);
    }
}

fn draw_sheets_icon(surface: &mut [u32], sw: usize, sh: usize, x: i32, y: i32) {
    fill_rect(surface, sw, sh, x, y, 12, 12, Color::rgb(34, 197, 94));
    fill_rect(surface, sw, sh, x + 2, y + 3, 8, 1, Color::WHITE);
    fill_rect(surface, sw, sh, x + 2, y + 6, 8, 1, Color::WHITE);
    fill_rect(surface, sw, sh, x + 2, y + 9, 8, 1, Color::WHITE);
}

fn draw_close_x(surface: &mut [u32], sw: usize, sh: usize, cx: i32, cy: i32, color: Color) {
    for d in 0..7 {
        fill_rect(surface, sw, sh, cx - 3 + d, cy - 3 + d, 2, 1, color);
        fill_rect(surface, sw, sh, cx - 3 + d, cy + 3 - d, 2, 1, color);
    }
}
