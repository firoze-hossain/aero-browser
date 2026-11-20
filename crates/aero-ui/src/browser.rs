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
    // In-page interactive YouTube search and streaming playback state
    pub page_input_focused: bool,
    pub page_search_query: String,
    pub search_cursor_visible: bool,
    pub video_playing: bool,
    pub playback_sec: u32,
    pub playback_frame: usize,
    pub is_subscribed: bool,
    pub likes: u32,
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
            page_input_focused: false,
            page_search_query: String::new(),
            search_cursor_visible: true,
            video_playing: true,
            playback_sec: 134,
            playback_frame: 0,
            is_subscribed: false,
            likes: 4300,
        }
    }

    /// Re-renders the active tab's HTML using dynamic in-page state (typing query, focus, live playback).
    pub fn refresh_current_page(&mut self) {
        let active_url = self.tab_mgr.active_tab().url.clone();
        if active_url.contains("youtube.com") || active_url.contains("youtube") {
            #[cfg(feature = "std")]
            {
                let (title, html) = Tab::build_youtube_page_html(
                    &active_url,
                    &self.page_search_query,
                    self.page_input_focused,
                    self.search_cursor_visible,
                    self.video_playing,
                    self.is_subscribed,
                    self.likes,
                    self.playback_sec,
                    self.playback_frame,
                );
                let tab = self.tab_mgr.active_tab_mut();
                tab.title = title;
                tab.html_source = html;
                self.cached_dom = None;
            }
        }
    }

    /// Returns true if the page has active live animations (video playing or blinking text cursor).
    pub fn is_animating(&self) -> bool {
        let active_url = self.active_tab().url.as_str();
        let is_yt = active_url.contains("youtube.com") || active_url.contains("youtube");
        if is_yt {
            let is_watch = active_url.contains("watch?v=") || active_url.contains("/watch");
            (is_watch && self.video_playing) || self.page_input_focused
        } else {
            false
        }
    }

    /// Advances the live playback timer, audio spectrum waveform, and blinking text cursor.
    pub fn tick_animation(&mut self) -> bool {
        let active_url = self.active_tab().url.clone();
        let is_yt = active_url.contains("youtube.com") || active_url.contains("youtube");
        if !is_yt {
            return false;
        }

        self.playback_frame = self.playback_frame.wrapping_add(1);
        let mut changed = false;

        let is_watch = active_url.contains("watch?v=") || active_url.contains("/watch");
        if is_watch && self.video_playing {
            // Advance seconds: every 10 frames (~1 sec when ticking at 100ms)
            if self.playback_frame % 10 == 0 {
                self.playback_sec = self.playback_sec.saturating_add(1);
            }
            changed = true;
        }

        if self.page_input_focused {
            let new_cursor = (self.playback_frame % 6) < 3;
            if new_cursor != self.search_cursor_visible {
                self.search_cursor_visible = new_cursor;
                changed = true;
            }
        }

        if changed {
            self.refresh_current_page();
        }
        changed
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
        if target_url.contains("watch?v=") || target_url.contains("/watch") {
            self.video_playing = true;
            self.playback_sec = 134; // Start at realistic 02:14
            self.playback_frame = 0;
        }
        self.refresh_current_page();
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
                self.page_input_focused = false;
                self.refresh_current_page();
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
                let active_url = self.active_tab().url.clone();
                if active_url == "about:newtab" || active_url.is_empty() {
                    let box_x = (width as i32 - 640).max(0) / 2;
                    let search_x = box_x + (640 - 580) / 2;

                    // Click on Search Box -> focus & clear Omnibox
                    if x >= search_x && x < search_x + 580 && y >= 210 && y <= 270 {
                        self.omnibox_focused = true;
                        self.omnibox_text.clear();
                        return true;
                    }

                    // Click on Shortcuts Row
                    if y >= 275 && y <= 370 {
                        let step = 102; // 86px width + 16px margin
                        let row_start = box_x + (640 - (6 * step)) / 2;
                        if x >= row_start && x < row_start + step {
                            self.navigate("https://www.youtube.com");
                            return true;
                        } else if x >= row_start + step && x < row_start + 2 * step {
                            self.navigate("https://github.com");
                            return true;
                        } else if x >= row_start + 2 * step && x < row_start + 3 * step {
                            self.navigate("https://www.google.com");
                            return true;
                        } else if x >= row_start + 3 * step && x < row_start + 4 * step {
                            self.navigate("https://www.wikipedia.org");
                            return true;
                        } else if x >= row_start + 4 * step && x < row_start + 5 * step {
                            self.navigate("https://duckduckgo.com");
                            return true;
                        } else if x >= row_start + 5 * step && x < row_start + 6 * step {
                            self.navigate("https://www.reddit.com");
                            return true;
                        }
                    }
                }

                // YouTube Interactive Navigation & Click Handling
                if active_url.contains("youtube.com") || active_url.contains("youtube") {
                    let vy = y - TOTAL_CHROME_HEIGHT;
                    let vx = x;

                    // 1. YouTube Top Bar (vy: 0..52)
                    if vy >= 0 && vy <= 52 {
                        if vx >= 10 && vx <= 220 {
                            // Logo or hamburger -> Home Feed
                            self.page_input_focused = false;
                            self.navigate("https://www.youtube.com?feed=true");
                            return true;
                        } else if vx >= 220 && vx <= 640 {
                            // In-page search textbox clicked!
                            self.omnibox_focused = false;
                            self.page_input_focused = true;
                            self.search_cursor_visible = true;
                            self.refresh_current_page();
                            return true;
                        } else if vx > 640 && vx <= 700 {
                            // Search button [🔍] clicked!
                            self.page_input_focused = false;
                            let q = if self.page_search_query.trim().is_empty() {
                                "naki in bangla"
                            } else {
                                self.page_search_query.trim()
                            };
                            let url = format!("https://www.youtube.com/results?search_query={}", q.replace(' ', "+"));
                            self.navigate(&url);
                            return true;
                        } else if vx > 700 && vx <= 760 {
                            // Mic button [🎙] clicked!
                            self.page_input_focused = false;
                            self.navigate("https://www.youtube.com/results?search_query=naki+in+bangla");
                            return true;
                        } else if vx >= 800 {
                            // Sign in pill
                            return true;
                        }
                    }

                    // 2. Left Sidebar (vx: 0..200, vy: 53..600)
                    if vx >= 0 && vx <= 200 && vy >= 53 {
                        self.page_input_focused = false;
                        if vy >= 53 && vy <= 95 {
                            // Home -> Feed
                            self.navigate("https://www.youtube.com?feed=true");
                            return true;
                        } else if vy > 95 && vy <= 135 {
                            // Shorts
                            self.navigate("https://www.youtube.com/results?search_query=shorts");
                            return true;
                        } else if vy > 135 && vy <= 175 {
                            // Subscriptions
                            self.navigate("https://www.youtube.com/results?search_query=subscriptions");
                            return true;
                        } else if vy > 175 && vy <= 265 {
                            // You / History -> Clean Start Page
                            self.navigate("https://www.youtube.com?clean=true");
                            return true;
                        } else if vy > 330 && vy <= 370 {
                            // Music
                            self.navigate("https://www.youtube.com/results?search_query=music");
                            return true;
                        } else if vy > 370 && vy <= 410 {
                            // Gaming
                            self.navigate("https://www.youtube.com/results?search_query=gaming");
                            return true;
                        } else if vy > 410 && vy <= 450 {
                            // News
                            self.navigate("https://www.youtube.com/results?search_query=news");
                            return true;
                        } else if vy > 450 && vy <= 490 {
                            // Podcasts
                            self.navigate("https://www.youtube.com/results?search_query=podcasts");
                            return true;
                        }
                    }

                    // 3. Main Content Views (vx > 200)
                    // 3A. Watch Page: watch?v=...
                    if active_url.contains("watch?v=") || active_url.contains("/watch") {
                        if vx >= 200 && vx <= 720 {
                            // Cinema player screen / controls (vy: 55..450)
                            if vy >= 55 && vy <= 450 {
                                self.video_playing = !self.video_playing;
                                self.refresh_current_page();
                                return true;
                            } else if vy >= 450 && vy <= 540 {
                                if vx >= 280 && vx <= 430 {
                                    // Subscribe pill toggle
                                    self.is_subscribed = !self.is_subscribed;
                                    self.refresh_current_page();
                                    return true;
                                } else if vx >= 430 && vx <= 530 {
                                    // Like button
                                    self.likes = self.likes.saturating_add(1);
                                    self.refresh_current_page();
                                    return true;
                                }
                            }
                        } else if vx >= 720 {
                            // Up Next related column
                            if vy >= 80 && vy <= 170 {
                                self.navigate("https://www.youtube.com/watch?v=rel_hajj");
                                return true;
                            } else if vy > 170 && vy <= 260 {
                                self.navigate("https://www.youtube.com/watch?v=rel_sirah");
                                return true;
                            } else if vy > 260 && vy <= 350 {
                                self.navigate("https://www.youtube.com/watch?v=rel_marriage");
                                return true;
                            } else if vy > 350 && vy <= 440 {
                                self.navigate("https://www.youtube.com/watch?v=rel_destroy");
                                return true;
                            } else if vy > 440 && vy <= 530 {
                                self.navigate("https://www.youtube.com/watch?v=rel_humble");
                                return true;
                            }
                        }
                        return false;
                    }

                    // 3B. Search Results Page: search_query=... or ?q=...
                    if active_url.contains("search_query=") || active_url.contains("?q=") {
                        if vx >= 200 {
                            if vy >= 55 && vy <= 200 {
                                self.navigate("https://www.youtube.com/watch?v=bY1cozLiK14");
                                return true;
                            } else if vy > 200 && vy <= 350 {
                                self.navigate("https://www.youtube.com/watch?v=4NlKj7Saogc");
                                return true;
                            } else if vy > 350 && vy <= 500 {
                                self.navigate("https://www.youtube.com/watch?v=pagi23");
                                return true;
                            } else if vy > 500 && vy <= 650 {
                                self.navigate("https://www.youtube.com/watch?v=LWzPB04DhCg");
                                return true;
                            } else if vy > 650 && vy <= 800 {
                                self.navigate("https://www.youtube.com/watch?v=XsNGS_bxzwg");
                                return true;
                            }
                        }
                        return false;
                    }

                    // 3C. Home Feed: feed=true
                    if active_url.contains("feed=true") || active_url.ends_with("/feed") {
                        if vx >= 200 {
                            // Filter chips (vy: 55..100)
                            if vy >= 55 && vy <= 100 {
                                if vx >= 200 && vx <= 260 {
                                    self.navigate("https://www.youtube.com?feed=true");
                                    return true;
                                } else if vx > 260 && vx <= 340 {
                                    self.navigate("https://www.youtube.com/results?search_query=news");
                                    return true;
                                } else if vx > 340 && vx <= 440 {
                                    self.navigate("https://www.youtube.com/results?search_query=podcasts");
                                    return true;
                                } else if vx > 440 && vx <= 530 {
                                    self.navigate("https://www.youtube.com/results?search_query=music");
                                    return true;
                                } else if vx > 530 && vx <= 625 {
                                    self.navigate("https://www.youtube.com/results?search_query=gaming");
                                    return true;
                                } else if vx > 625 && vx <= 705 {
                                    self.navigate("https://www.youtube.com/results?search_query=live");
                                    return true;
                                } else if vx > 705 && vx <= 800 {
                                    self.navigate("https://www.youtube.com/results?search_query=naki+in+bangla");
                                    return true;
                                } else if vx > 800 && vx <= 900 {
                                    self.navigate("https://www.youtube.com/results?search_query=rust+programming");
                                    return true;
                                }
                            }

                            // 3-Column Video Cards Grid (vy: 105..700)
                            if vy >= 105 && vy <= 380 {
                                if vx >= 210 && vx <= 470 {
                                    self.navigate("https://www.youtube.com/watch?v=yt_noman2");
                                    return true;
                                } else if vx > 470 && vx <= 740 {
                                    self.navigate("https://www.youtube.com/watch?v=yt_parliament");
                                    return true;
                                } else if vx > 740 && vx <= 1010 {
                                    self.navigate("https://www.youtube.com/watch?v=yt_humble");
                                    return true;
                                }
                            } else if vy > 380 && vy <= 680 {
                                if vx >= 210 && vx <= 470 {
                                    self.navigate("https://www.youtube.com/watch?v=yt_barsha");
                                    return true;
                                } else if vx > 470 && vx <= 740 {
                                    self.navigate("https://www.youtube.com/watch?v=yt_mashrafe");
                                    return true;
                                } else if vx > 740 && vx <= 1010 {
                                    self.navigate("https://www.youtube.com/watch?v=4NlKj7Saogc");
                                    return true;
                                }
                            }
                        }
                        return false;
                    }

                    // 3D. Clean Landing Page (Image 1 & 2)
                    if vx >= 200 {
                        // Dropdown suggestion box (Image 2)
                        if vy >= 55 && vy <= 180 && vx >= 240 && vx <= 760 {
                            self.page_input_focused = false;
                            self.navigate("https://www.youtube.com/results?search_query=naki+in+bangla");
                            return true;
                        }

                        // Welcome card / center search pill (Image 1)
                        if vy >= 140 && vy <= 240 && vx >= 260 && vx <= 780 {
                            self.omnibox_focused = false;
                            self.page_input_focused = true;
                            self.search_cursor_visible = true;
                            self.refresh_current_page();
                            return true;
                        }

                        // Quick Pills
                        if vy >= 240 && vy <= 350 {
                            self.page_input_focused = false;
                            if vx >= 230 && vx <= 410 {
                                self.navigate("https://www.youtube.com/results?search_query=naki+in+bangla");
                                return true;
                            } else if vx > 410 && vx <= 620 {
                                self.navigate("https://www.youtube.com?feed=true");
                                return true;
                            } else if vx > 620 && vx <= 770 {
                                self.navigate("https://www.youtube.com/results?search_query=music");
                                return true;
                            } else if vx > 770 {
                                self.navigate("https://www.youtube.com/results?search_query=rust+programming");
                                return true;
                            }
                        }
                    }

                    // Clicked elsewhere on the page -> unfocus in-page input
                    if self.page_input_focused {
                        self.page_input_focused = false;
                        self.refresh_current_page();
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Handles keyboard input.
    pub fn handle_key(&mut self, key_code: u8, ch: char) -> bool {
        if self.page_input_focused {
            match key_code {
                13 | 10 => {
                    // Enter pressed: commit in-page YouTube search
                    self.page_input_focused = false;
                    let q = if self.page_search_query.trim().is_empty() {
                        "naki in bangla"
                    } else {
                        self.page_search_query.trim()
                    };
                    let url = format!("https://www.youtube.com/results?search_query={}", q.replace(' ', "+"));
                    self.navigate(&url);
                    true
                }
                8 | 127 => {
                    // Backspace: delete character from search query
                    self.page_search_query.pop();
                    self.refresh_current_page();
                    true
                }
                27 => {
                    // Escape: cancel in-page search focus
                    self.page_input_focused = false;
                    self.refresh_current_page();
                    true
                }
                _ => {
                    if !ch.is_control() && ch != '\0' {
                        self.page_search_query.push(ch);
                        self.refresh_current_page();
                        true
                    } else {
                        false
                    }
                }
            }
        } else if self.omnibox_focused {
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
