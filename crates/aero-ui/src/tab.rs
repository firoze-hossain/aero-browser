//! # Aero Tab Manager
//!
//! Handles multi-tab session state, navigation history stack, scroll offsets,
//! and page loading lifecycle for Aero Browser.

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

use aero_engine::net::{InternalPages, Url};
use aero_engine::shields::ShieldStats;

/// A single browser tab instance.
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub html_source: String,
    pub scroll_y: f32,
    pub max_scroll: f32,
    pub is_loading: bool,
    pub history: Vec<String>,
    pub history_index: usize,
    pub reader_mode: bool,
}

impl Tab {
    pub fn new(id: usize, url: &str, stats: &ShieldStats) -> Self {
        let (title, html) = Self::load_page(url, stats);
        Tab {
            id,
            title,
            url: url.to_owned(),
            html_source: html,
            scroll_y: 0.0,
            max_scroll: 0.0,
            is_loading: false,
            history: vec![url.to_owned()],
            history_index: 0,
            reader_mode: false,
        }
    }

    pub fn navigate(&mut self, url: &str, stats: &ShieldStats) {
        let (title, html) = Self::load_page(url, stats);
        self.url = url.to_owned();
        self.title = title;
        self.html_source = html;
        self.scroll_y = 0.0;
        self.reader_mode = false;

        // Truncate forward history if navigating to new URL
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(url.to_owned());
        self.history_index = self.history.len() - 1;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self, stats: &ShieldStats) -> bool {
        if self.can_go_back() {
            self.history_index -= 1;
            let url = self.history[self.history_index].clone();
            let (title, html) = Self::load_page(&url, stats);
            self.url = url;
            self.title = title;
            self.html_source = html;
            self.scroll_y = 0.0;
            self.reader_mode = false;
            true
        } else {
            false
        }
    }

    pub fn go_forward(&mut self, stats: &ShieldStats) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            let url = self.history[self.history_index].clone();
            let (title, html) = Self::load_page(&url, stats);
            self.url = url;
            self.title = title;
            self.html_source = html;
            self.scroll_y = 0.0;
            self.reader_mode = false;
            true
        } else {
            false
        }
    }

    pub fn reload(&mut self, stats: &ShieldStats) {
        let (title, html) = Self::load_page(&self.url, stats);
        self.title = title;
        self.html_source = html;
    }

    pub fn toggle_reader_mode(&mut self) {
        self.reader_mode = !self.reader_mode;
    }

    #[cfg(feature = "std")]
    fn fetch_live_web(url: &str) -> Option<(String, String)> {
        let mut target = url.to_owned();
        if !target.starts_with("http://") && !target.starts_with("https://") {
            target = format!("https://{}", target);
        }

        let resp = minreq::get(&target)
            .with_timeout(5)
            .with_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) Aero/1.0.0 (Pure-Rust Engine; Brave Shields)")
            .send()
            .ok()?;

        if resp.status_code >= 200 && resp.status_code < 400 {
            let body = resp.as_str().ok()?.to_owned();
            let title = Self::extract_title_from_html(&body).unwrap_or_else(|| {
                Url::parse(&target).map(|u| u.host).unwrap_or_else(|| "Aero Web".to_owned())
            });
            Some((title, body))
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    fn extract_title_from_html(html: &str) -> Option<String> {
        let lower = html.to_lowercase();
        let start_tag = "<title>";
        let end_tag = "</title>";
        if let Some(start_idx) = lower.find(start_tag) {
            if let Some(end_idx) = lower[start_idx..].find(end_tag) {
                let raw_title = &html[start_idx + start_tag.len()..start_idx + end_idx];
                return Some(raw_title.trim().to_owned());
            }
        }
        None
    }

    fn load_page(url_str: &str, stats: &ShieldStats) -> (String, String) {
        let trimmed = url_str.trim();

        if trimmed == "about:newtab" || trimmed.is_empty() || trimmed == "about:blank" {
            ("New Tab".to_owned(), InternalPages::new_tab(stats))
        } else if trimmed == "about:shields" {
            ("Aero Shields".to_owned(), InternalPages::shields_page(stats))
        } else if trimmed == "about:version" {
            ("About Aero".to_owned(), InternalPages::version_page())
        } else if trimmed.contains("youtube.com") || trimmed.contains("youtube") {
            ("YouTube".to_owned(), InternalPages::youtube_portal())
        } else if trimmed.contains("github.com") || trimmed.contains("github") {
            ("firoze-hossain / novaos · GitHub".to_owned(), InternalPages::github_portal())
        } else if trimmed.contains("wikipedia.org") || trimmed.contains("wikipedia") {
            ("Operating system - Wikipedia".to_owned(), InternalPages::wikipedia_portal())
        } else if trimmed.contains("duckduckgo.com") || trimmed.contains("duckduckgo") {
            ("DuckDuckGo — Privacy, simplified.".to_owned(), InternalPages::duckduckgo_portal("NovaOS and Aero Browser"))
        } else if trimmed.contains("news.ycombinator.com") || trimmed.contains("hackernews") {
            ("Hacker News".to_owned(), InternalPages::hackernews_portal())
        } else if trimmed.starts_with("https://search.") || trimmed.contains("search?q=") {
            let q = trimmed.split("search?q=").nth(1)
                .or_else(|| trimmed.split("?q=").nth(1))
                .unwrap_or("Web Search");
            ("Search Results - Aero".to_owned(), InternalPages::duckduckgo_portal(q))
        } else if trimmed.contains("novaos.org") || trimmed.contains("nova") {
            (
                "NovaOS - The Modern Rust Operating System".to_owned(),
                "<!DOCTYPE html>
<html>
<head>
<title>NovaOS Official Portal</title>
<style>
body { background-color: #0b0f19; color: #f1f5f9; padding: 24px; font-family: sans-serif; }
.hero { max-width: 800px; margin: 0 auto; }
h1 { color: #38bdf8; font-size: 32px; }
.card { background-color: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 18px; margin: 16px 0; }
.btn { background-color: #ff5500; color: #fff; padding: 8px 16px; border-radius: 6px; text-decoration: none; display: inline-block; }
</style>
</head>
<body>
<div class=\"hero\">
    <h1>NovaOS Enterprise Edition</h1>
    <p>A next-generation hybrid kernel written in pure, memory-safe Rust with preemptive SMP and rich graphical userland.</p>
    <div class=\"card\">
        <h3>Default Web Browser: Aero</h3>
        <p>Aero is powered by an independent, zero-dependency layout engine with built-in Brave privacy shields.</p>
        <button class=\"btn\">Explore Documentation</button>
    </div>
</div>
</body>
</html>".to_owned()
            )
        } else if trimmed.contains("rust-lang.org") || trimmed.contains("rust") {
            (
                "Rust Programming Language".to_owned(),
                "<!DOCTYPE html>
<html>
<head>
<title>Rust Programming Language</title>
<style>
body { background-color: #0f172a; color: #f1f5f9; padding: 24px; font-family: sans-serif; }
.container { max-width: 780px; margin: 0 auto; }
h1 { color: #f97316; }
.box { background-color: #1e293b; border-radius: 8px; padding: 16px; margin: 12px 0; }
</style>
</head>
<body>
<div class=\"container\">
    <h1>Rust: Empowering Everyone to Build Reliable Software</h1>
    <p>Performance, reliability, productivity. Pick all three.</p>
    <div class=\"box\">
        <h3>Zero-Cost Abstractions</h3>
        <p>Aero Browser layout and painting engines are written entirely in 100% safe Rust without Chromium or WebKit.</p>
    </div>
</div>
</body>
</html>".to_owned()
            )
        } else {
            // Attempt live network fetch via HTTP/HTTPS if running with std
            #[cfg(feature = "std")]
            {
                if let Some((live_title, live_html)) = Self::fetch_live_web(trimmed) {
                    return (live_title, live_html);
                }
            }

            // Fallback for offline, unreachable hosts or bare-metal environment
            let title = if let Some(url) = Url::parse(trimmed) {
                format!("{} - Aero Browser", url.host)
            } else {
                "Unable to Connect - Aero".to_owned()
            };
            (title, InternalPages::error_page(trimmed, "Could not establish a connection to this server. Ensure the web address is correct or verify network connectivity."))
        }
    }
}

/// Manages multiple tabs in an Aero browser session.
#[derive(Debug, Clone)]
pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new(stats: &ShieldStats) -> Self {
        let initial_tab = Tab::new(1, "about:newtab", stats);
        TabManager {
            tabs: vec![initial_tab],
            active_index: 0,
            next_id: 2,
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }

    pub fn add_tab(&mut self, url: &str, stats: &ShieldStats) -> usize {
        let new_tab = Tab::new(self.next_id, url, stats);
        self.next_id += 1;
        self.tabs.push(new_tab);
        self.active_index = self.tabs.len() - 1;
        self.active_index
    }

    pub fn new_tab(&mut self, url: &str, stats: &ShieldStats) -> usize {
        self.add_tab(url, stats)
    }

    pub fn navigate_active(&mut self, url: &str, stats: &ShieldStats) {
        self.active_tab_mut().navigate(url, stats);
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            // Keep at least one tab open
            return;
        }
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
        }
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }
}
