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
}
