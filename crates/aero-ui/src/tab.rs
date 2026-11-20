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
            .with_timeout(8)
            .with_max_redirects(5)
            .with_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
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
    fn fetch_live_youtube(query: Option<&str>) -> Option<(String, String)> {
        let endpoint = if let Some(q) = query {
            let encoded = q.replace(' ', "+");
            format!("https://www.youtube.com/results?search_query={}", encoded)
        } else {
            "https://www.youtube.com/results?search_query=trending".to_owned()
        };

        let resp = minreq::get(&endpoint)
            .with_timeout(8)
            .with_max_redirects(5)
            .with_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .ok()?;

        if resp.status_code >= 200 && resp.status_code < 400 {
            let body = resp.as_str().ok()?;
            let videos = Self::parse_youtube_videos_from_html(body);
            let display_q = query.unwrap_or("Trending");
            let html = Self::build_youtube_feed_html(display_q, &videos);
            let title = if let Some(q) = query {
                format!("{} - YouTube", q)
            } else {
                "YouTube".to_owned()
            };
            Some((title, html))
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    fn parse_youtube_videos_from_html(html: &str) -> Vec<(String, String, String, String)> {
        let mut results = Vec::new();
        let mut cursor = 0;
        while let Some(vr_idx) = html[cursor..].find("\"videoRenderer\":{") {
            let abs_idx = cursor + vr_idx;
            cursor = abs_idx + 17;

            let end_idx = (cursor + 3500).min(html.len());
            let chunk = &html[cursor..end_idx];

            let title = if let Some(t_idx) = chunk.find("\"title\":{\"runs\":[{\"text\":\"") {
                let t_start = t_idx + 26;
                if let Some(t_end) = chunk[t_start..].find("\"}") {
                    chunk[t_start..t_start + t_end].to_owned()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let channel = if let Some(c_idx) = chunk.find("\"shortBylineText\":{\"runs\":[{\"text\":\"") {
                let c_start = c_idx + 37;
                if let Some(c_end) = chunk[c_start..].find("\"") {
                    chunk[c_start..c_start + c_end].to_owned()
                } else {
                    "Creator".to_owned()
                }
            } else {
                "Creator".to_owned()
            };

            let views = if let Some(v_idx) = chunk.find("\"viewCountText\":{\"simpleText\":\"") {
                let v_start = v_idx + 31;
                if let Some(v_end) = chunk[v_start..].find("\"") {
                    chunk[v_start..v_start + v_end].to_owned()
                } else {
                    "Popular".to_owned()
                }
            } else {
                "Popular".to_owned()
            };

            let duration = if let Some(d_idx) = chunk.find("\"lengthText\":{\"simpleText\":\"") {
                let d_start = d_idx + 28;
                if let Some(d_end) = chunk[d_start..].find("\"") {
                    chunk[d_start..d_start + d_end].to_owned()
                } else {
                    "Video".to_owned()
                }
            } else {
                "Video".to_owned()
            };

            results.push((title, channel, views, duration));
            if results.len() >= 9 {
                break;
            }
        }
        results
    }

    #[cfg(feature = "std")]
    fn build_youtube_feed_html(query: &str, videos: &[(String, String, String, String)]) -> String {
        let mut cards = String::new();
        for (title, channel, views, dur) in videos {
            cards.push_str(&format!(
                "<div class=\"video-card\">
                    <div class=\"thumb-box\">
                        <div class=\"dur-pill\">{}</div>
                        <div style=\"color: #ff0000; font-size: 15px; font-weight: bold; padding-top: 40px; text-align: center;\">[>] Watch</div>
                    </div>
                    <div class=\"vid-title\">{}</div>
                    <div class=\"vid-channel\">{}</div>
                    <div class=\"vid-meta\">{}</div>
                </div>",
                dur, title, channel, views
            ));
        }

        if cards.is_empty() {
            cards.push_str("<div style=\"padding: 40px; color: #888; text-align: center;\">No videos found. Try a different search query.</div>");
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - YouTube</title>
<style>
body {{
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.yt-header {{
    background-color: #0f0f0f;
    border-bottom: 1px solid #272727;
    padding: 12px 24px;
    display: block;
}}
.yt-logo {{
    display: inline-block;
    color: #ff0000;
    font-size: 22px;
    font-weight: bold;
    width: 140px;
}}
.yt-search-bar {{
    display: inline-block;
    background-color: #121212;
    border: 1px solid #303030;
    border-radius: 20px;
    padding: 8px 18px;
    width: 500px;
    color: #f1f1f1;
    font-size: 14px;
    margin: 0 16px;
}}
.yt-feed {{
    padding: 20px 24px;
    max-width: 1280px;
    margin: 0 auto;
}}
.video-card {{
    display: inline-block;
    width: 360px;
    margin: 12px 14px;
    background-color: #1a1a1a;
    border: 1px solid #272727;
    border-radius: 10px;
    padding: 12px;
}}
.thumb-box {{
    background-color: #212121;
    border-radius: 8px;
    height: 140px;
    margin-bottom: 10px;
    padding: 8px;
}}
.dur-pill {{
    background-color: #000000;
    color: #ffffff;
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 10px;
    float: right;
}}
.vid-title {{
    font-size: 14px;
    font-weight: bold;
    color: #ffffff;
    margin-bottom: 6px;
    display: block;
}}
.vid-channel {{
    font-size: 12px;
    color: #aaaaaa;
    margin-bottom: 4px;
    display: block;
}}
.vid-meta {{
    font-size: 11px;
    color: #717171;
    display: block;
}}
</style>
</head>
<body>
<div class=\"yt-header\">
    <div class=\"yt-logo\">[>] YouTube</div>
    <div class=\"yt-search-bar\">Search: {}</div>
</div>

<div class=\"yt-feed\">
    {}
</div>
</body>
</html>",
            query, query, cards
        )
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
            #[cfg(feature = "std")]
            {
                let query = if trimmed.contains("search_query=") {
                    trimmed.split("search_query=").nth(1)
                } else if trimmed.contains("?q=") {
                    trimmed.split("?q=").nth(1)
                } else {
                    None
                };
                if let Some((yt_title, yt_html)) = Self::fetch_live_youtube(query) {
                    return (yt_title, yt_html);
                }
            }
            ("YouTube".to_owned(), InternalPages::youtube_portal())
        } else {
            // Live network fetch via HTTP/HTTPS for all domains (GitHub, Wikipedia, Google, etc.)
            #[cfg(feature = "std")]
            {
                if let Some((live_title, live_html)) = Self::fetch_live_web(trimmed) {
                    return (live_title, live_html);
                }
            }

            // Fallback for offline or unreachable hosts
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
