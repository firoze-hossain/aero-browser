//! # Aero Browser Controller
//!
//! Orchestrates the multi-tab manager, Omnibox, navigation controls,
//! Brave Shields privacy flyout, Chrome DevTools inspector, and layout renderer.

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
use aero_engine::dom::Node;
use aero_engine::html::HtmlParser;
use aero_engine::net::Url;
use aero_engine::render_html_to_surface;
use aero_engine::shields::ShieldEngine;

use crate::bookmarks::BookmarkManager;
use crate::chrome::{
    hit_test_chrome, render_chrome, ChromeClickTarget, BOOKMARKS_HEIGHT, NAVBAR_HEIGHT,
    TAB_STRIP_HEIGHT, TOTAL_CHROME_HEIGHT,
};
use crate::devtools::{render_devtools_panel, DevToolsState, DevToolsTab};
use crate::shields_panel::render_shields_flyout;
use crate::tab::{Tab, TabManager};

/// The primary Aero Browser state controller.
pub struct AeroBrowser {
    pub tab_mgr: TabManager,
    pub bookmarks: BookmarkManager,
    pub shields: ShieldEngine,
    pub devtools: DevToolsState,
    pub show_shields_flyout: bool,
    pub omnibox_text: String,
    pub omnibox_focused: bool,
    pub cached_dom: Option<Node>,
}

impl AeroBrowser {
    /// Initializes a new browser session with default homepage tab and bookmarks.
    pub fn new() -> Self {
        let shields = ShieldEngine::new();
        let tab_mgr = TabManager::new(&shields.stats);
        let bookmarks = BookmarkManager::new();
        let devtools = DevToolsState::default();
        let omnibox_text = tab_mgr.active_tab().url.clone();

        AeroBrowser {
            tab_mgr,
            bookmarks,
            shields,
            devtools,
            show_shields_flyout: false,
            omnibox_text,
            omnibox_focused: false,
            cached_dom: None,
        }
    }

    /// Returns a reference to the active tab.
    pub fn active_tab(&self) -> &Tab {
        self.tab_mgr.active_tab()
    }

    /// Returns a mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        self.tab_mgr.active_tab_mut()
    }

    /// Navigates the current active tab to a new URL.
    pub fn navigate(&mut self, url: &str) {
        let domain = Url::parse(url).map(|u| u.host).unwrap_or_default();
        let action = self.shields.evaluate_url(url, &domain);
        let target_url = match action {
            aero_engine::shields::UrlAction::Allow(u) => u,
            aero_engine::shields::UrlAction::UpgradeHttps(u) => u,
            aero_engine::shields::UrlAction::Block { reason } => {
                format!("about:shields?blocked={}", reason)
            }
        };

        self.tab_mgr.navigate_active(&target_url, &self.shields.stats);
        self.omnibox_text = self.tab_mgr.active_tab().url.clone();
        self.cached_dom = None;
    }

    /// Navigates back in history for the active tab.
    pub fn go_back(&mut self) -> bool {
        let stats = &self.shields.stats;
        let res = self.tab_mgr.active_tab_mut().go_back(stats);
        if res {
            self.omnibox_text = self.tab_mgr.active_tab().url.clone();
            self.cached_dom = None;
        }
        res
    }

    /// Navigates forward in history for the active tab.
    pub fn go_forward(&mut self) -> bool {
        let stats = &self.shields.stats;
        let res = self.tab_mgr.active_tab_mut().go_forward(stats);
        if res {
            self.omnibox_text = self.tab_mgr.active_tab().url.clone();
            self.cached_dom = None;
        }
        res
    }

    /// Opens a new tab with given URL.
    pub fn new_tab(&mut self, url: &str) -> usize {
        let idx = self.tab_mgr.new_tab(url, &self.shields.stats);
        self.omnibox_text = self.tab_mgr.active_tab().url.clone();
        self.cached_dom = None;
        idx
    }

    /// Handles mouse clicks at `(x, y)` inside the browser window.
    pub fn handle_click(&mut self, x: i32, y: i32, width: usize, height: usize) -> bool {
        let w = width as i32;
        let h = height as i32;

        // 1. If Shields Flyout is open, hit-test against it first
        if self.show_shields_flyout {
            let panel_w = 320;
            let panel_h = 290;
            let px = (w - panel_w - 14).max(10);
            let py = TOTAL_CHROME_HEIGHT + 2;

            if x >= px && x < px + panel_w && y >= py && y < py + panel_h {
                // Check if the toggle pill button was clicked
                let toggle_btn_w = 80;
                let toggle_btn_h = 24;
                let toggle_x = px + panel_w - toggle_btn_w - 14;
                let toggle_y = py + 12;

                if x >= toggle_x && x < toggle_x + toggle_btn_w && y >= toggle_y && y < toggle_y + toggle_btn_h {
                    let domain = Url::parse(&self.active_tab().url)
                        .map(|u| u.host)
                        .unwrap_or_default();
                    self.shields.toggle_shields_for_domain(&domain);
                }
                return true;
            } else {
                // Clicked outside shields flyout -> dismiss it
                self.show_shields_flyout = false;
            }
        }

        // 2. DevTools click test if open
        let devtools_h = if self.devtools.is_open { 180 } else { 0 };
        let devtools_y = h - devtools_h;

        if self.devtools.is_open && y >= devtools_y {
            if y < devtools_y + 28 {
                // DevTools header tabs
                let tabs = [
                    ("Elements", DevToolsTab::Elements),
                    ("Console", DevToolsTab::Console),
                    ("Network", DevToolsTab::Network),
                    ("Shields", DevToolsTab::Shields),
                ];
                let mut tab_x = 12;
                for (_, tab_enum) in &tabs {
                    if x >= tab_x && x < tab_x + 65 {
                        self.devtools.active_tab = *tab_enum;
                        return true;
                    }
                    tab_x += 70;
                }
            }
            return true;
        }

        // 3. Browser Chrome click targets (Tabs, Omnibox, Navigation, Bookmarks)
        let target = hit_test_chrome(x, y, width, &self.tab_mgr, &self.bookmarks);
        match target {
            ChromeClickTarget::TabSelect(idx) => {
                self.tab_mgr.active_index = idx;
                self.omnibox_text = self.tab_mgr.active_tab().url.clone();
                self.cached_dom = None;
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::TabClose(idx) => {
                self.tab_mgr.close_tab(idx);
                self.omnibox_text = self.tab_mgr.active_tab().url.clone();
                self.cached_dom = None;
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::NewTab => {
                self.new_tab("about:newtab");
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::NavBack => {
                self.go_back();
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::NavForward => {
                self.go_forward();
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::NavReload => {
                let url = self.tab_mgr.active_tab().url.clone();
                self.navigate(&url);
                true
            }
            ChromeClickTarget::NavHome => {
                self.navigate("about:newtab");
                true
            }
            ChromeClickTarget::OmniboxFocus => {
                self.omnibox_focused = true;
                true
            }
            ChromeClickTarget::ToggleShields => {
                self.show_shields_flyout = !self.show_shields_flyout;
                true
            }
            ChromeClickTarget::ToggleReader => {
                self.tab_mgr.active_tab_mut().toggle_reader_mode();
                self.cached_dom = None;
                true
            }
            ChromeClickTarget::ToggleDevTools => {
                self.devtools.is_open = !self.devtools.is_open;
                true
            }
            ChromeClickTarget::BookmarkClick(url) => {
                self.navigate(&url);
                self.omnibox_focused = false;
                true
            }
            ChromeClickTarget::None => {
                // Click was in viewport
                self.omnibox_focused = false;
                false
            }
        }
    }

    /// Handles keyboard input.
    pub fn handle_key(&mut self, key_code: u8, ch: char) -> bool {
        if self.omnibox_focused {
            match key_code {
                13 | 10 => {
                    // Enter pressed: commit navigation
                    self.omnibox_focused = false;
                    let input = self.omnibox_text.trim().to_owned();
                    if !input.is_empty() {
                        let final_url = if input.starts_with("about:")
                            || input.starts_with("http://")
                            || input.starts_with("https://")
                            || input.starts_with("file://")
                        {
                            input
                        } else if input.contains('.') && !input.contains(' ') {
                            format!("https://{}", input)
                        } else {
                            // Default search query
                            format!("https://search.brave.com/search?q={}", input)
                        };
                        self.navigate(&final_url);
                    }
                    true
                }
                8 | 127 => {
                    // Backspace
                    self.omnibox_text.pop();
                    true
                }
                27 => {
                    // Escape: cancel edit
                    self.omnibox_focused = false;
                    self.omnibox_text = self.tab_mgr.active_tab().url.clone();
                    true
                }
                _ => {
                    if !ch.is_control() {
                        self.omnibox_text.push(ch);
                        true
                    } else {
                        false
                    }
                }
            }
        } else {
            // Viewport scrolling hotkeys
            match key_code {
                38 | b'k' => {
                    // Up arrow
                    self.handle_scroll(-40.0)
                }
                40 | b'j' => {
                    // Down arrow
                    self.handle_scroll(40.0)
                }
                33 => {
                    // Page Up
                    self.handle_scroll(-200.0)
                }
                34 | b' ' => {
                    // Page Down or Space
                    self.handle_scroll(200.0)
                }
                _ => false,
            }
        }
    }

    /// Handles scroll wheel or key navigation.
    pub fn handle_scroll(&mut self, delta_y: f32) -> bool {
        let max_scroll = self.tab_mgr.active_tab().max_scroll;
        let tab = self.tab_mgr.active_tab_mut();
        let new_scroll = (tab.scroll_y + delta_y).clamp(0.0, max_scroll);
        if (new_scroll - tab.scroll_y).abs() > 0.01 {
            tab.scroll_y = new_scroll;
            true
        } else {
            false
        }
    }

    /// Renders the complete browser interface (Chrome, Page Viewport, DevTools, Shields Flyout)
    /// into a 32-bit XRGB frame buffer surface.
    pub fn render(&mut self, surface: &mut [u32], width: usize, height: usize) {
        if width == 0 || height == 0 || surface.len() < width * height {
            return;
        }

        // 1. Render Chrome Banner (Tabs, Nav Bar, Omnibox, Bookmarks)
        render_chrome(
            surface,
            width,
            height,
            &self.tab_mgr,
            &self.bookmarks,
            self.show_shields_flyout,
            self.shields.stats.trackers_blocked,
            &self.omnibox_text,
            self.omnibox_focused,
        );

        // 2. Compute Viewport dimensions
        let chrome_h = TOTAL_CHROME_HEIGHT as usize;
        let devtools_h = if self.devtools.is_open { 180 } else { 0 };
        let vp_h = height.saturating_sub(chrome_h + devtools_h);

        if vp_h > 0 {
            let vp_offset = chrome_h * width;
            let vp_len = vp_h * width;

            if surface.len() >= vp_offset + vp_len {
                let vp_slice = &mut surface[vp_offset..vp_offset + vp_len];

                // Render page HTML into viewport slice
                let active_tab = self.tab_mgr.active_tab();
                let (content_height, _) = render_html_to_surface(
                    &active_tab.html_source,
                    vp_slice,
                    width,
                    vp_h,
                    active_tab.scroll_y,
                );

                // Update tab scroll boundaries
                self.tab_mgr.active_tab_mut().max_scroll = (content_height - vp_h as f32).max(0.0);
            }
        }

        // 3. Render Chrome DevTools Inspector if open
        if self.devtools.is_open && devtools_h > 0 {
            let devtools_y = (height - devtools_h) as i32;
            let dom = self.cached_dom.get_or_insert_with(|| {
                HtmlParser::parse(&self.tab_mgr.active_tab().html_source)
            });
            render_devtools_panel(
                surface,
                width,
                height,
                0,
                devtools_y,
                width as i32,
                devtools_h as i32,
                &self.devtools,
                dom,
            );
        }

        // 4. Render Brave Shields Flyout drop-down if active
        if self.show_shields_flyout {
            let domain = Url::parse(&self.active_tab().url)
                .map(|u| u.host)
                .unwrap_or_default();
            let panel_w = 320;
            let px = (width as i32 - panel_w - 14).max(10);
            let py = TOTAL_CHROME_HEIGHT + 2;

            render_shields_flyout(surface, width, height, px, py, &domain, &self.shields);
        }
    }
}
