//! # Aero UI
//!
//! Sleek, modern browser interface components synthesizing the best UX patterns
//! from Brave, Google Chrome, and Mozilla Firefox:
//! - Chrome: rounded tab strip, Omnibox with live lock & star icons, split DevTools DOM inspector
//! - Brave: Lion Shields privacy button, flyout panel with per-site blocking toggles and stats
//! - Firefox: distraction-free Reader View extractor and theme switcher, bookmarks quick-bar

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod bookmarks;
pub mod browser;
pub mod chrome;
pub mod devtools;
pub mod shields_panel;
pub mod tab;

pub use bookmarks::{Bookmark, BookmarkManager};
pub use browser::AeroBrowser;
pub use chrome::{
    hit_test_chrome, render_chrome, ChromeClickTarget, BOOKMARKS_HEIGHT, NAVBAR_HEIGHT,
    TAB_STRIP_HEIGHT, TOTAL_CHROME_HEIGHT,
};
pub use devtools::{render_devtools_panel, DevToolsState, DevToolsTab};
pub use shields_panel::render_shields_flyout;
pub use tab::{Tab, TabManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_initialization() {
        let browser = AeroBrowser::new();
        assert_eq!(browser.tab_mgr.tabs.len(), 1);
        assert_eq!(browser.active_tab().url, "about:newtab");
        assert!(!browser.bookmarks.items.is_empty());
        assert!(!browser.devtools.is_open);
        assert!(!browser.show_shields_flyout);
    }

    #[test]
    fn test_tab_navigation_and_history() {
        let mut browser = AeroBrowser::new();
        assert_eq!(browser.active_tab().url, "about:newtab");

        browser.navigate("about:shields");
        assert_eq!(browser.active_tab().url, "about:shields");
        assert!(browser.active_tab().can_go_back());

        browser.go_back();
        assert_eq!(browser.active_tab().url, "about:newtab");
        assert!(browser.active_tab().can_go_forward());

        browser.go_forward();
        assert_eq!(browser.active_tab().url, "about:shields");
    }

    #[test]
    fn test_multi_tab_operations() {
        let mut browser = AeroBrowser::new();
        assert_eq!(browser.tab_mgr.tabs.len(), 1);

        // Open new tab
        browser.tab_mgr.new_tab("about:version", &browser.shields.stats);
        assert_eq!(browser.tab_mgr.tabs.len(), 2);
        assert_eq!(browser.tab_mgr.active_index, 1);
        assert_eq!(browser.active_tab().url, "about:version");

        // Close second tab
        browser.tab_mgr.close_tab(1);
        assert_eq!(browser.tab_mgr.tabs.len(), 1);
        assert_eq!(browser.tab_mgr.active_index, 0);
    }

    #[test]
    fn test_omnibox_input_and_navigation() {
        let mut browser = AeroBrowser::new();
        browser.omnibox_focused = true;
        browser.omnibox_text = "about:".to_string();

        browser.handle_key(0, 'b');
        browser.handle_key(0, 'l');
        browser.handle_key(0, 'a');
        browser.handle_key(0, 'n');
        browser.handle_key(0, 'k');
        assert_eq!(browser.omnibox_text, "about:blank");

        // Commit with Enter (key code 13)
        browser.handle_key(13, '\n');
        assert_eq!(browser.active_tab().url, "about:blank");
        assert!(!browser.omnibox_focused);
    }

    #[test]
    fn test_full_browser_frame_rendering() {
        let mut browser = AeroBrowser::new();
        let width = 800;
        let height = 600;
        let mut surface = vec![0u32; width * height];

        // 1. Initial render
        browser.render(&mut surface, width, height);
        let non_zero_pixels = surface.iter().filter(|&&p| p != 0).count();
        assert!(non_zero_pixels > 1000);

        // 2. Open shields flyout and re-render
        browser.show_shields_flyout = true;
        browser.render(&mut surface, width, height);

        // 3. Open devtools and re-render
        browser.devtools.is_open = true;
        browser.render(&mut surface, width, height);
    }

    #[test]
    fn test_youtube_search_typing_and_execution() {
        let mut browser = AeroBrowser::new();
        browser.navigate("https://www.youtube.com");
        assert_eq!(browser.active_tab().url, "https://www.youtube.com");

        // Click search box area to focus in-page search
        let handled = browser.handle_click(300, 110, 1024, 720);
        assert!(handled);
        assert!(browser.page_input_focused);
        assert!(!browser.omnibox_focused);

        // Type "vlad"
        browser.handle_key(0, 'v');
        browser.handle_key(0, 'l');
        browser.handle_key(0, 'a');
        browser.handle_key(0, 'd');
        assert_eq!(browser.page_search_query, "vlad");

        // Backspace
        browser.handle_key(8, '\x08');
        assert_eq!(browser.page_search_query, "vla");

        // Commit search with Enter
        browser.handle_key(13, '\n');
        assert!(!browser.page_input_focused);
        assert_eq!(browser.active_tab().url, "https://www.youtube.com/results?search_query=vla");
        assert!(browser.active_tab().html_source.contains("Vlad"));
    }

    #[test]
    fn test_youtube_video_streaming_and_playback_ticking() {
        let mut browser = AeroBrowser::new();
        browser.navigate("https://www.youtube.com/watch?v=bY1cozLiK14");

        assert!(browser.video_playing);
        assert_eq!(browser.playback_sec, 134); // 02:14
        assert!(browser.is_animating());

        // Simulate 20 animation ticks (2 seconds)
        for _ in 0..20 {
            browser.tick_animation();
        }

        assert_eq!(browser.playback_sec, 136); // 02:16
        assert!(browser.active_tab().html_source.contains("02:16"));
        assert!(browser.active_tab().html_source.contains("SPECTRUM:"));

        // Toggle play/pause
        browser.video_playing = false;
        browser.refresh_current_page();
        assert!(browser.active_tab().html_source.contains("PAUSED"));
    }

    #[test]
    fn test_youtube_video_thematic_metadata_and_cinema_stage() {
        let v_vlad = Tab::get_video_by_id("bY1cozLiK14");
        assert!(v_vlad.title.contains("Vlad and Niki"));
        let (_, _, hud, _, tag, main, _, _, _, _, speaker) = Tab::get_player_stage_details(&v_vlad);
        assert!(hud.contains("4K UHD"));
        assert!(tag.contains("ICE CREAM"));
        assert!(main.contains("VLAD"));
        assert!(speaker.contains("VLAD"));

        let v_waqas = Tab::get_video_by_id("pagi23");
        assert!(v_waqas.title.contains("Pagol Naki"));
        let (_, _, hud_w, _, tag_w, main_w, _, _, _, _, speaker_w) = Tab::get_player_stage_details(&v_waqas);
        assert!(hud_w.contains("1080p"));
        assert!(tag_w.contains("STUDIO"));
        assert!(main_w.contains("PAGOL NAKI"));
        assert!(speaker_w.contains("WAQAS"));

        let v_parliament = Tab::get_video_by_id("yt_parliament");
        assert!(v_parliament.title.contains("Parliament"));
        let (_, _, hud_p, _, tag_p, main_p, _, _, _, _, speaker_p) = Tab::get_player_stage_details(&v_parliament);
        assert!(hud_p.contains("1080p60"));
        assert!(tag_p.contains("PARLIAMENT"));
        assert!(main_p.contains("MUGDHO PURO SANGSAD"));
        assert!(speaker_p.contains("ATN"));
    }

    #[test]
    fn test_print_feed_layout_cards() {
        let (_title, html) = Tab::build_youtube_page_html("https://www.youtube.com?feed=true", "", false, false, true, false, 4300, 134, 0);
        let mut surface = vec![0u32; 1024 * 768];
        let (_height, dl) = aero_engine::render_html_to_surface(&html, &mut surface, 1024, 768, 0.0);
        for (i, item) in dl.items.iter().enumerate() {
            let s = format!("{:?}", item);
            if s.contains("Chupchap") || s.contains("Motiversity") || s.contains("MB") {
                println!("Item #{}: {:?}", i, item);
            }
        }
    }

    #[test]
    fn test_bookmark_folders_and_dropdown_navigation() {
        let mut browser = AeroBrowser::new();

        // 1. Verify default bookmark folders matching user's Brave bar
        let f_7th = browser.bookmarks.items.iter().find(|b| b.title() == "7th Dream Pr...").unwrap();
        assert!(f_7th.is_folder());
        let children = f_7th.children().unwrap();
        assert!(children.iter().any(|c| c.title().contains("Google Sheets")));
        assert!(children.iter().any(|c| c.title() == "novaos"));
        assert!(children.iter().any(|c| c.title() == "roze"));
        assert!(children.iter().any(|c| c.title() == "stratosdb"));

        // 2. Click folder to open dropdown
        let folder_idx = browser.bookmarks.items.iter().position(|b| b.title() == "7th Dream Pr...").unwrap();
        browser.open_bookmark_folder = Some(folder_idx);

        // 3. Render frame with open folder dropdown
        let mut surface = vec![0u32; 1024 * 720];
        browser.render(&mut surface, 1024, 720);
        let non_zero = surface.iter().filter(|&&p| p != 0).count();
        assert!(non_zero > 1000);

        // 4. Hit-test click inside dropdown on NovaOS_Build_Plan.xlsx
        // dd_x is calculated from folder position, dd_y is TOTAL_CHROME_HEIGHT
        let mut fx = 12;
        for (i, bm) in browser.bookmarks.items.iter().enumerate() {
            if i == folder_idx { break; }
            fx += (bm.title().len() * 8) as i32 + 28;
        }
        let dd_w = 320;
        let dd_x = fx.min(1024 - dd_w - 8).max(8);
        let dd_y = TOTAL_CHROME_HEIGHT;
        // Second item is NovaOS_Build_Plan.xlsx (y = dd_y + 6 + 30 = dd_y + 36)
        let clicked = browser.handle_click(dd_x + 50, dd_y + 40, 1024, 720);
        assert!(clicked);
        assert_eq!(browser.active_tab().url, "https://docs.google.com/spreadsheets/d/novaos_build_plan");
        assert!(browser.open_bookmark_folder.is_none());
    }

    #[test]
    fn test_main_toggle_menu_and_more_tools_submenu() {
        let mut browser = AeroBrowser::new();

        // 1. Click top-right hamburger toggle button [≡]
        let clicked = browser.handle_click(1024 - 20, 50, 1024, 720);
        assert!(clicked);
        assert!(browser.show_main_menu);

        // 2. Click "More tools >" in the main menu
        // menu_x = 1024 - 280 - 10 = 734
        // More tools is row 18 (y ~= TOTAL_CHROME_HEIGHT - 4 + 8 + 18 * 24 + dividers)
        browser.show_more_tools_submenu = true;

        // 3. Click "Developer tools" from submenu
        let sub_w = 260;
        let menu_x = 1024 - 350 - 12;
        let sub_x = menu_x - sub_w - 4;
        let sub_y = TOTAL_CHROME_HEIGHT + 236;
        // DevTools is at rel_y ~245
        let dev_clicked = browser.handle_click(sub_x + 50, sub_y + 245, 1024, 720);
        assert!(dev_clicked);
        assert!(browser.devtools.is_open);
        assert!(!browser.show_main_menu);
        assert!(!browser.show_more_tools_submenu);

        // 4. Open main menu again and click "Settings"
        browser.show_main_menu = true;
        let menu_y = TOTAL_CHROME_HEIGHT - 4;
        // Settings is at rel_y ~510
        let settings_clicked = browser.handle_click(menu_x + 50, menu_y + 510, 1024, 720);
        assert!(settings_clicked);
        assert_eq!(browser.active_tab().url, "about:settings");
        assert!(!browser.show_main_menu);
    }

    #[test]
    fn test_settings_page_rendering_and_category_navigation() {
        let mut browser = AeroBrowser::new();
        browser.navigate("about:settings");

        assert_eq!(browser.active_tab().url, "about:settings");
        let html = &browser.active_tab().html_source;
        assert!(html.contains("Get started"));
        assert!(html.contains("Appearance"));
        assert!(html.contains("Shields"));
        assert!(html.contains("Sync your settings across devices"));
        assert!(html.contains("Profile 1"));
        assert!(html.contains("Make Aero default browser"));
        assert!(html.contains("On startup"));
        assert!(html.contains("Dashboard"));

        // Click sidebar category: Appearance (index 1: y = TOTAL_CHROME_HEIGHT + 56 + 32)
        let clicked = browser.handle_click(80, TOTAL_CHROME_HEIGHT + 56 + 36, 1024, 720);
        assert!(clicked);
        assert_eq!(browser.active_tab().url, "about:settings?category=appearance");
        assert!(browser.active_tab().html_source.contains("Aero Theme & Colors"));

        // Click sidebar category: Shields (index 3: y = TOTAL_CHROME_HEIGHT + 56 + 3 * 32)
        let clicked_shields = browser.handle_click(80, TOTAL_CHROME_HEIGHT + 56 + 100, 1024, 720);
        assert!(clicked_shields);
        assert_eq!(browser.active_tab().url, "about:settings?category=shields");
        assert!(browser.active_tab().html_source.contains("Aero Brave Shields Engine"));

        // Click sidebar category: About Aero (index 16)
        let clicked_about = browser.handle_click(80, TOTAL_CHROME_HEIGHT + 56 + 16 * 32 + 10, 1024, 720);
        assert!(clicked_about);
        assert_eq!(browser.active_tab().url, "about:settings?category=about");
        assert!(browser.active_tab().html_source.contains("About Aero Browser"));
        assert!(browser.active_tab().html_source.contains("Multi-Paradigm Engine Architecture"));
    }
}
