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
        let (hud, tag, main, _bullets, speaker, _, _, _, _) = Tab::get_player_stage_details(&v_vlad);
        assert!(hud.contains("4K UHD"));
        assert!(tag.contains("KIDS ADVENTURE"));
        assert!(main.contains("Ice Cream Truck"));
        assert!(speaker.contains("VLAD"));

        let v_waqas = Tab::get_video_by_id("pagi23");
        assert!(v_waqas.title.contains("Pagol Naki"));
        let (_, tag_w, main_w, _, speaker_w, _, _, _, _) = Tab::get_player_stage_details(&v_waqas);
        assert!(tag_w.contains("OFFICIAL ACOUSTIC"));
        assert!(main_w.contains("Pagol Naki"));
        assert!(speaker_w.contains("WAQAS"));

        let v_natok = Tab::get_video_by_id("LWzPB04DhCg");
        assert!(v_natok.title.contains("Vul Naki Prem"));
        let (_, tag_n, main_n, _, speaker_n, _, _, _, _) = Tab::get_player_stage_details(&v_natok);
        assert!(tag_n.contains("BANGLA NATOK"));
        assert!(main_n.contains("Vul Naki Prem"));
        assert!(speaker_n.contains("AROSH KHAN"));
    }
}
