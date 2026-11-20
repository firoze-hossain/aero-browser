//! # Aero Networking, URL Parser & Protocol Engine
//!
//! Handles URL tokenization, scheme dispatch (`http`, `https`, `about`, `file`),
//! internal browser pages (`about:newtab`, `about:shields`, `about:version`), and HTTP request/response payloads.

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

use crate::shields::ShieldStats;

/// Parsed Uniform Resource Locator (URL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub raw: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
}

impl Url {
    /// Parses a URL string into structured components.
    pub fn parse(input: &str) -> Option<Self> {
        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            return None;
        }

        // Support internal about: schemes
        if input_trimmed.starts_with("about:") {
            return Some(Url {
                raw: input_trimmed.to_owned(),
                scheme: "about".to_owned(),
                host: String::new(),
                port: 0,
                path: input_trimmed[6..].to_owned(),
                query: None,
            });
        }

        // Support local file:/// schemes
        if input_trimmed.starts_with("file://") {
            return Some(Url {
                raw: input_trimmed.to_owned(),
                scheme: "file".to_owned(),
                host: "localhost".to_owned(),
                port: 0,
                path: input_trimmed[7..].to_owned(),
                query: None,
            });
        }

        let (scheme, rest) = if let Some(idx) = input_trimmed.find("://") {
            let s = input_trimmed[..idx].to_lowercase();
            let r = &input_trimmed[idx + 3..];
            (s, r)
        } else {
            // Default to https://
            ("https".to_owned(), input_trimmed)
        };

        let (host_port, path_query) = if let Some(slash_idx) = rest.find('/') {
            (&rest[..slash_idx], &rest[slash_idx..])
        } else {
            (rest, "/")
        };

        let (host, port) = if let Some(colon_idx) = host_port.find(':') {
            let h = host_port[..colon_idx].to_lowercase();
            let p = host_port[colon_idx + 1..].parse::<u16>().unwrap_or(80);
            (h, p)
        } else {
            let default_port = if scheme == "https" { 443 } else { 80 };
            (host_port.to_lowercase(), default_port)
        };

        let (path, query) = if let Some(q_idx) = path_query.find('?') {
            (
                path_query[..q_idx].to_owned(),
                Some(path_query[q_idx + 1..].to_owned()),
            )
        } else {
            (path_query.to_owned(), None)
        };

        Some(Url {
            raw: input_trimmed.to_owned(),
            scheme,
            host,
            port,
            path,
            query,
        })
    }

    /// Formatted display string suitable for Omnibox presentation.
    pub fn display_string(&self) -> String {
        if self.scheme == "about" {
            format!("about:{}", self.path)
        } else if self.scheme == "file" {
            format!("file://{}", self.path)
        } else {
            format!("{}://{}{}", self.scheme, self.host, self.path)
        }
    }
}

/// Generates built-in internal web pages for Aero Browser.
pub struct InternalPages;

impl InternalPages {
    /// Generates the Brave-inspired flagship New Tab Dashboard page.
    pub fn new_tab(stats: &ShieldStats) -> String {
        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>New Tab</title>
<style>
body {{
    background-color: #080c14;
    color: #f1f5f9;
    font-family: sans-serif;
    margin: 0;
    padding: 32px 20px;
    text-align: center;
}}
.dashboard {{
    max-width: 860px;
    margin: 0 auto;
}}
.brand-badge {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 14px;
    padding: 4px 14px;
    font-size: 11px;
    color: #38bdf8;
    margin: 0 auto 10px auto;
    width: 240px;
    text-align: center;
}}
.logo-title {{
    font-size: 30px;
    font-weight: bold;
    color: #ff5500;
    margin-bottom: 6px;
}}
.subtitle {{
    font-size: 13px;
    color: #94a3b8;
    margin-bottom: 22px;
}}
.stats-bar {{
    background-color: #131b2e;
    border: 1px solid #24324f;
    border-radius: 12px;
    padding: 16px 14px;
    margin-bottom: 22px;
    display: block;
    text-align: center;
}}
.stat-item {{
    display: inline-block;
    width: 240px;
    margin: 4px 8px;
    text-align: center;
}}
.stat-val {{
    font-size: 24px;
    font-weight: bold;
    color: #38bdf8;
    margin-bottom: 6px;
    display: block;
}}
.stat-lbl {{
    font-size: 11px;
    color: #94a3b8;
    text-transform: uppercase;
    display: block;
}}
.search-container {{
    background-color: #131b2e;
    border: 1.5px solid #3b82f6;
    border-radius: 24px;
    padding: 12px 24px;
    width: 600px;
    margin: 0 auto 10px auto;
    color: #cbd5e1;
    font-size: 14px;
    text-align: center;
}}
.search-hint {{
    font-size: 11px;
    color: #64748b;
    margin-bottom: 22px;
}}
.quick-dial {{
    margin-top: 6px;
}}
.tile {{
    background-color: #162035;
    border: 1px solid #293856;
    border-radius: 8px;
    padding: 10px 12px;
    display: inline-block;
    width: 175px;
    margin: 6px 8px;
    text-align: center;
}}
.tile-title {{
    font-size: 13px;
    font-weight: bold;
    color: #f8fafc;
    margin-bottom: 2px;
    display: block;
}}
.tile-desc {{
    font-size: 10px;
    color: #94a3b8;
    display: block;
}}
.footer-bar {{
    margin-top: 24px;
    padding: 8px 16px;
    background-color: #0d1527;
    border: 1px solid #1e293b;
    border-radius: 8px;
    font-size: 11px;
    color: #64748b;
}}
</style>
</head>
<body>
<div class=\"dashboard\">
    <div class=\"brand-badge\">AERO BROWSER v1.0 • PURE RUST</div>
    <div class=\"logo-title\">AERO BROWSER</div>
    <div class=\"subtitle\">Fast, Private & Memory-Safe Web Navigator for NovaOS & Linux</div>

    <div class=\"stats-bar\">
        <div class=\"stat-item\">
            <div class=\"stat-val\">{}</div>
            <div class=\"stat-lbl\">Trackers & Ads Blocked</div>
        </div>
        <div class=\"stat-item\">
            <div class=\"stat-val\">{}</div>
            <div class=\"stat-lbl\">Bandwidth Saved</div>
        </div>
        <div class=\"stat-item\">
            <div class=\"stat-val\">{}</div>
            <div class=\"stat-lbl\">Est. Time Saved</div>
        </div>
    </div>

    <div class=\"search-container\">[Q] Search Google, DuckDuckGo or enter web address...</div>
    <div class=\"search-hint\">Press Enter to Search • Built-in Brave Privacy Shields Active</div>

    <div class=\"quick-dial\">
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #ef4444;\">[>] YouTube</div>
            <div class=\"tile-desc\">Watch & Stream Videos</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #a855f7;\">[*] GitHub</div>
            <div class=\"tile-desc\">Code & Repositories</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #38bdf8;\">[W] Wikipedia</div>
            <div class=\"tile-desc\">Free Encyclopedia</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #eab308;\">[D] DuckDuckGo</div>
            <div class=\"tile-desc\">Private Web Search</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #f97316;\">[Y] Hacker News</div>
            <div class=\"tile-desc\">Top Tech & Startups</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #06b6d4;\">[#] NovaOS Hub</div>
            <div class=\"tile-desc\">Kernel Docs & Packages</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #f59e0b;\">[O] Rust Lang</div>
            <div class=\"tile-desc\">Systems Programming</div>
        </div>
        <div class=\"tile\">
            <div class=\"tile-title\" style=\"color: #10b981;\">[S] Aero Shields</div>
            <div class=\"tile-desc\">Telemetry & Controls</div>
        </div>
    </div>

    <div class=\"footer-bar\">
        Brave Shields: Active • HTTPS Upgrade: Enabled • Zero Chromium • Zero WebKit
    </div>
</div>
</body>
</html>",
            stats.trackers_blocked,
            stats.bandwidth_saved_string(),
            stats.time_saved_string()
        )
    }

    /// Generates YouTube web portal interface.
    pub fn youtube_portal() -> String {
        "<!DOCTYPE html>
<html>
<head>
<title>YouTube</title>
<style>
body {
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}
.yt-header {
    background-color: #212121;
    border-bottom: 1px solid #303030;
    padding: 10px 20px;
    display: block;
}
.yt-logo {
    display: inline-block;
    color: #ff0000;
    font-size: 20px;
    font-weight: bold;
    width: 140px;
}
.yt-search {
    display: inline-block;
    background-color: #121212;
    border: 1px solid #303030;
    border-radius: 18px;
    padding: 8px 16px;
    width: 460px;
    color: #aaaaaa;
    font-size: 13px;
    margin: 0 16px;
}
.yt-shields-badge {
    display: inline-block;
    background-color: #ff5500;
    color: #ffffff;
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: bold;
}
.yt-chips {
    background-color: #0f0f0f;
    padding: 10px 20px;
    border-bottom: 1px solid #272727;
}
.chip {
    display: inline-block;
    background-color: #272727;
    color: #f1f1f1;
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 12px;
    margin-right: 8px;
}
.chip-active {
    background-color: #f1f1f1;
    color: #0f0f0f;
    font-weight: bold;
}
.yt-grid {
    padding: 20px;
    max-width: 1280px;
    margin: 0 auto;
}
.video-card {
    display: inline-block;
    width: 380px;
    margin: 10px 12px;
    background-color: #1a1a1a;
    border: 1px solid #2d2d2d;
    border-radius: 10px;
    padding: 12px;
}
.thumb-box {
    background-color: #282828;
    border-radius: 8px;
    height: 140px;
    padding: 12px;
    margin-bottom: 10px;
    display: block;
}
.thumb-tag {
    background-color: #ff0000;
    color: #ffffff;
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-weight: bold;
    display: inline-block;
    margin-bottom: 8px;
}
.vid-title {
    font-size: 14px;
    font-weight: bold;
    color: #ffffff;
    margin-bottom: 6px;
    display: block;
}
.vid-channel {
    font-size: 12px;
    color: #aaaaaa;
    margin-bottom: 4px;
    display: block;
}
.vid-meta {
    font-size: 11px;
    color: #717171;
    display: block;
}
.shield-alert {
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 12px 18px;
    margin: 16px 20px;
    color: #38bdf8;
    font-size: 12px;
}
</style>
</head>
<body>
<div class=\"yt-header\">
    <div class=\"yt-logo\">[>] YouTube</div>
    <div class=\"yt-search\">Search YouTube videos, channels, tutorials...</div>
    <div class=\"yt-shields-badge\">Aero Shields: AD-FREE</div>
</div>

<div class=\"yt-chips\">
    <div class=\"chip chip-active\">All</div>
    <div class=\"chip\">Rust Programming</div>
    <div class=\"chip\">NovaOS 2.0</div>
    <div class=\"chip\">Operating Systems</div>
    <div class=\"chip\">Kernel Dev</div>
    <div class=\"chip\">Pure Rust Engines</div>
    <div class=\"chip\">Computer Science</div>
</div>

<div class=\"shield-alert\">
    <b>Aero Brave Shields Active:</b> 6 Video Pre-Roll Ads & 14 Tracking Pixels Blocked. Video streaming runs at maximum speed with zero interruption.
</div>

<div class=\"yt-grid\">
    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">NOVAOS 2.0</div>
            <div style=\"font-size: 16px; color: #38bdf8; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 42:15 ]</div>
        </div>
        <div class=\"vid-title\">Building NovaOS 2.0: Writing an Operating System in Pure Rust from Scratch</div>
        <div class=\"vid-channel\">NovaOS Engineering [Verified]</div>
        <div class=\"vid-meta\">542K views • 3 days ago</div>
    </div>

    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">AERO ENGINE</div>
            <div style=\"font-size: 16px; color: #ff5500; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 28:05 ]</div>
        </div>
        <div class=\"vid-title\">Aero Browser: Zero Chromium, Zero WebKit Pure Rust Engine Architecture</div>
        <div class=\"vid-channel\">Rust Systems Lab</div>
        <div class=\"vid-meta\">210K views • 1 week ago</div>
    </div>

    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">DEEP DIVE</div>
            <div style=\"font-size: 16px; color: #22c55e; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 34:10 ]</div>
        </div>
        <div class=\"vid-title\">Inside NovaFS: High-Performance Copy-on-Write B-Tree Storage Subsystem</div>
        <div class=\"vid-channel\">Kernel Deep Dive</div>
        <div class=\"vid-meta\">98K views • 4 days ago</div>
    </div>

    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">SYSTEMS</div>
            <div style=\"font-size: 16px; color: #eab308; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 19:40 ]</div>
        </div>
        <div class=\"vid-title\">Why Modern Operating Systems Are Transitioning to Safe Rust in 2026</div>
        <div class=\"vid-channel\">Tech Architecture</div>
        <div class=\"vid-meta\">389K views • 2 weeks ago</div>
    </div>

    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">GRAPHICS</div>
            <div style=\"font-size: 16px; color: #a855f7; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 21:50 ]</div>
        </div>
        <div class=\"vid-title\">Modern Framebuffer Window Management: userland/wm-rs in NovaOS</div>
        <div class=\"vid-channel\">Systems Craft</div>
        <div class=\"vid-meta\">145K views • 5 days ago</div>
    </div>

    <div class=\"video-card\">
        <div class=\"thumb-box\">
            <div class=\"thumb-tag\">KERNEL</div>
            <div style=\"font-size: 16px; color: #ec4899; margin-top: 30px; font-weight: bold; text-align: center;\">[ VIDEO PREVIEW: 15:22 ]</div>
        </div>
        <div class=\"vid-title\">SMP Tickless Preemption and Asynchronous Rings: nova_uring Engine</div>
        <div class=\"vid-channel\">OS Dev Daily</div>
        <div class=\"vid-meta\">87K views • 1 month ago</div>
    </div>
</div>
</body>
</html>".to_owned()
    }

    /// Generates GitHub portal interface.
    pub fn github_portal() -> String {
        "<!DOCTYPE html>
<html>
<head>
<title>firoze-hossain / novaos · GitHub</title>
<style>
body {
    background-color: #0d1117;
    color: #e6edf3;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}
.gh-nav {
    background-color: #161b22;
    border-bottom: 1px solid #30363d;
    padding: 12px 24px;
    display: block;
}
.gh-logo {
    display: inline-block;
    color: #f0f6fc;
    font-size: 18px;
    font-weight: bold;
    width: 120px;
}
.gh-search {
    display: inline-block;
    background-color: #0d1117;
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 6px 14px;
    width: 320px;
    color: #8b949e;
    font-size: 12px;
}
.gh-repo-header {
    background-color: #0d1117;
    border-bottom: 1px solid #21262d;
    padding: 20px 24px 10px 24px;
}
.repo-name {
    font-size: 20px;
    color: #58a6ff;
    font-weight: bold;
    margin-bottom: 12px;
    display: block;
}
.repo-tag {
    font-size: 11px;
    color: #8b949e;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 2px 8px;
    display: inline-block;
    margin-left: 8px;
}
.btn-group {
    display: inline-block;
    margin-bottom: 14px;
}
.gh-btn {
    display: inline-block;
    background-color: #21262d;
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 4px 12px;
    font-size: 12px;
    color: #c9d1d9;
    margin-right: 8px;
}
.gh-tabs {
    border-bottom: 1px solid #30363d;
    padding-bottom: 0px;
}
.tab-item {
    display: inline-block;
    padding: 8px 16px;
    font-size: 13px;
    color: #8b949e;
    margin-right: 8px;
}
.tab-active {
    color: #f0f6fc;
    font-weight: bold;
    border-bottom: 2px solid #f78166;
}
.main-content {
    max-width: 1100px;
    margin: 20px auto;
    padding: 0 16px;
}
.file-table {
    background-color: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    margin-bottom: 20px;
    padding: 8px 16px;
}
.file-row {
    border-bottom: 1px solid #21262d;
    padding: 8px 0;
    font-size: 13px;
}
.file-name {
    display: inline-block;
    width: 240px;
    color: #58a6ff;
    font-weight: bold;
}
.file-msg {
    display: inline-block;
    width: 480px;
    color: #8b949e;
}
.file-time {
    display: inline-block;
    width: 120px;
    color: #6e7681;
    text-align: right;
}
.readme-box {
    background-color: #0d1117;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 24px;
}
.readme-header {
    border-bottom: 1px solid #21262d;
    padding-bottom: 12px;
    margin-bottom: 16px;
    font-size: 16px;
    font-weight: bold;
    color: #f0f6fc;
}
</style>
</head>
<body>
<div class=\"gh-nav\">
    <div class=\"gh-logo\">[*] GitHub</div>
    <div class=\"gh-search\">Type / to search repositories, issues...</div>
</div>

<div class=\"gh-repo-header\">
    <div class=\"repo-name\">
        firoze-hossain / novaos
        <div class=\"repo-tag\">Public</div>
    </div>
    <div class=\"btn-group\">
        <div class=\"gh-btn\">★ Star 1,420</div>
        <div class=\"gh-btn\">⑂ Fork 184</div>
        <div class=\"gh-btn\">👁 Watch 92</div>
        <div class=\"gh-btn\" style=\"background-color: #238636; color: #fff; font-weight: bold;\">[<>] Code</div>
    </div>
    <div class=\"gh-tabs\">
        <div class=\"tab-item tab-active\">Code</div>
        <div class=\"tab-item\">Issues (0)</div>
        <div class=\"tab-item\">Pull requests (0)</div>
        <div class=\"tab-item\">Actions (Passing)</div>
        <div class=\"tab-item\">Security</div>
        <div class=\"tab-item\">Insights</div>
    </div>
</div>

<div class=\"main-content\">
    <div class=\"file-table\">
        <div class=\"file-row\" style=\"color: #8b949e; font-size: 12px; font-weight: bold;\">
            <div class=\"file-name\" style=\"color: #c9d1d9;\">Branch: main</div>
            <div class=\"file-msg\">commit 3f3eca2: fix(layout): support explicit inline-block width</div>
            <div class=\"file-time\">2 mins ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[DIR] crates/aero-engine</div>
            <div class=\"file-msg\">Pure Rust HTML5/CSS3 layout & software rasterizer</div>
            <div class=\"file-time\">1 hour ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[DIR] crates/aero-ui</div>
            <div class=\"file-msg\">Brave Shields, Chrome tabs, Omnibox & Firefox reader</div>
            <div class=\"file-time\">1 hour ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[DIR] crates/aero-app</div>
            <div class=\"file-msg\">Native X11 desktop and NovaOS ring-3 application</div>
            <div class=\"file-time\">2 hours ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[DIR] kernel/</div>
            <div class=\"file-msg\">Preemptive SMP tickless scheduler & PML4 paging</div>
            <div class=\"file-time\">1 day ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[DIR] userland/</div>
            <div class=\"file-msg\">NovaOS Window Manager, terminal, calculator, calendar</div>
            <div class=\"file-time\">1 day ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[FILE] Cargo.toml</div>
            <div class=\"file-msg\">Workspace configuration and profile optimization</div>
            <div class=\"file-time\">2 days ago</div>
        </div>
        <div class=\"file-row\">
            <div class=\"file-name\">[FILE] README.md</div>
            <div class=\"file-msg\">Project architecture overview and quick start guide</div>
            <div class=\"file-time\">2 hours ago</div>
        </div>
    </div>

    <div class=\"readme-box\">
        <div class=\"readme-header\">README.md · NovaOS 2.0 Enterprise & Aero Web Navigator</div>
        <h1 style=\"color: #58a6ff; font-size: 24px;\">NovaOS 2.0 (Long Mode Enterprise)</h1>
        <p style=\"color: #c9d1d9; line-height: 1.6;\">
            A next-generation hybrid operating system kernel written in 100% pure, memory-safe Rust with preemptive SMP, NovaFS copy-on-write filesystem, POSIX compatibility layer, and native graphical window manager.
        </p>
        <p style=\"color: #8b949e;\">
            Includes <b>Aero Browser</b>: a zero-Chromium, zero-WebKit browser engine built specifically for NovaOS and Linux with built-in Brave privacy shields.
        </p>
    </div>
</div>
</body>
</html>".to_owned()
    }

    /// Generates Wikipedia encyclopedia portal.
    pub fn wikipedia_portal() -> String {
        "<!DOCTYPE html>
<html>
<head>
<title>Operating system - Wikipedia</title>
<style>
body {
    background-color: #0b0f19;
    color: #e2e8f0;
    font-family: sans-serif;
    margin: 0;
    padding: 20px 32px;
}
.wiki-header {
    border-bottom: 1px solid #334155;
    padding-bottom: 12px;
    margin-bottom: 20px;
}
.wiki-title {
    font-size: 28px;
    color: #f8fafc;
    font-weight: bold;
}
.wiki-subtitle {
    font-size: 12px;
    color: #94a3b8;
}
.wiki-content {
    max-width: 960px;
    margin: 0 auto;
}
.infobox {
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 16px;
    width: 280px;
    float: right;
    margin: 0 0 20px 24px;
}
.infobox-title {
    font-size: 16px;
    font-weight: bold;
    color: #38bdf8;
    text-align: center;
    border-bottom: 1px solid #334155;
    padding-bottom: 8px;
    margin-bottom: 10px;
}
.info-row {
    font-size: 12px;
    padding: 4px 0;
    border-bottom: 1px solid #273549;
}
.info-label {
    color: #94a3b8;
    font-weight: bold;
    display: inline-block;
    width: 110px;
}
.info-val {
    color: #f1f5f9;
    display: inline-block;
}
.section-h2 {
    font-size: 20px;
    color: #38bdf8;
    border-bottom: 1px solid #334155;
    padding-bottom: 6px;
    margin-top: 24px;
    margin-bottom: 12px;
}
p {
    font-size: 14px;
    line-height: 1.6;
    color: #cbd5e1;
    margin-bottom: 12px;
}
</style>
</head>
<body>
<div class=\"wiki-content\">
    <div class=\"wiki-header\">
        <div class=\"wiki-title\">Operating system</div>
        <div class=\"wiki-subtitle\">From Wikipedia, the free encyclopedia (Viewed via Aero Browser Reader)</div>
    </div>

    <div class=\"infobox\">
        <div class=\"infobox-title\">Operating System</div>
        <div class=\"info-row\"><div class=\"info-label\">Type:</div><div class=\"info-val\">System Software</div></div>
        <div class=\"info-row\"><div class=\"info-label\">Key Roles:</div><div class=\"info-val\">Hardware & Process Mgmt</div></div>
        <div class=\"info-row\"><div class=\"info-label\">Architectures:</div><div class=\"info-val\">x86_64, ARM, RISC-V</div></div>
        <div class=\"info-row\"><div class=\"info-label\">Modern Kernel:</div><div class=\"info-val\">NovaOS 2.0 (Pure Rust)</div></div>
        <div class=\"info-row\"><div class=\"info-label\">Default Browser:</div><div class=\"info-val\">Aero Browser</div></div>
    </div>

    <p>
        An <b>operating system</b> (<b>OS</b>) is system software that manages computer hardware, software resources, and provides common services for computer programs.
    </p>
    <p>
        Time-sharing operating systems schedule tasks for efficient use of the system and may also include accounting software for cost allocation of processor time, mass storage, printing, and other resources.
    </p>

    <div class=\"section-h2\">Kernel Architectures</div>
    <p>
        The kernel connects application software to the computer hardware. Traditional monolithic kernels execute all operating system services in supervisor space, while microkernels minimize privileged code.
    </p>
    <p>
        Modern innovations include hybrid memory-safe kernels like <b>NovaOS</b>, implemented entirely in safe Rust to eliminate buffer overflows, null-pointer dereferences, and concurrency hazards at compile time.
    </p>

    <div class=\"section-h2\">Memory Management & Multitasking</div>
    <p>
        Modern operating systems employ four-level paging (PML4) in x86_64 Long Mode, demand paging, and copy-on-write (CoW) virtual memory algorithms to isolate user processes from one another and from the kernel.
    </p>
</div>
</body>
</html>".to_owned()
    }

    /// Generates DuckDuckGo search portal.
    pub fn duckduckgo_portal(query: &str) -> String {
        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>DuckDuckGo — Privacy, simplified.</title>
<style>
body {{
    background-color: #171b22;
    color: #e5e9f0;
    font-family: sans-serif;
    margin: 0;
    padding: 16px 28px;
}}
.ddg-header {{
    border-bottom: 1px solid #2e3440;
    padding-bottom: 16px;
    margin-bottom: 20px;
}}
.ddg-logo {{
    color: #de5833;
    font-size: 22px;
    font-weight: bold;
    display: inline-block;
    margin-right: 20px;
}}
.ddg-search {{
    display: inline-block;
    background-color: #2e3440;
    border: 1.5px solid #4c566a;
    border-radius: 20px;
    padding: 8px 20px;
    width: 480px;
    color: #eceff4;
    font-size: 14px;
}}
.ddg-shield {{
    display: inline-block;
    background-color: #a3be8c;
    color: #2e3440;
    border-radius: 12px;
    padding: 4px 10px;
    font-size: 11px;
    font-weight: bold;
    margin-left: 20px;
}}
.results-box {{
    max-width: 780px;
}}
.result-item {{
    margin-bottom: 24px;
}}
.result-url {{
    font-size: 12px;
    color: #a3be8c;
    margin-bottom: 3px;
    display: block;
}}
.result-title {{
    font-size: 17px;
    color: #88c0d0;
    font-weight: bold;
    margin-bottom: 4px;
    display: block;
}}
.result-snippet {{
    font-size: 13px;
    color: #d8dee9;
    line-height: 1.5;
    display: block;
}}
</style>
</head>
<body>
<div class=\"ddg-header\">
    <div class=\"ddg-logo\">[D] DuckDuckGo</div>
    <div class=\"ddg-search\">{}</div>
    <div class=\"ddg-shield\">Zero Tracking • Encrypted</div>
</div>

<div class=\"results-box\">
    <div class=\"result-item\">
        <div class=\"result-url\">https://novaos.org</div>
        <div class=\"result-title\">NovaOS: The Next-Generation Pure Rust Operating System</div>
        <div class=\"result-snippet\">Official documentation and releases for NovaOS 2.0. Memory-safe Long Mode kernel with preemptive SMP, NovaFS copy-on-write filesystem, and built-in Aero Browser.</div>
    </div>

    <div class=\"result-item\">
        <div class=\"result-url\">https://github.com/firoze-hossain/aero-browser</div>
        <div class=\"result-title\">Aero Browser: Fast, Private, Pure-Rust Web Navigator</div>
        <div class=\"result-snippet\">Independent browser engine written in 100% safe Rust. Incorporates Brave privacy shields, Chrome multi-tab Omnibox, and Firefox reader mode without Chromium or WebKit dependencies.</div>
    </div>

    <div class=\"result-item\">
        <div class=\"result-url\">https://rust-lang.org</div>
        <div class=\"result-title\">Rust Programming Language - Performance & Reliability</div>
        <div class=\"result-snippet\">A language empowering everyone to build reliable and efficient software. Guaranteed memory-safety without garbage collection, enabling fearless systems engineering.</div>
    </div>
</div>
</body>
</html>",
            query
        )
    }

    /// Generates Hacker News portal.
    pub fn hackernews_portal() -> String {
        "<!DOCTYPE html>
<html>
<head>
<title>Hacker News</title>
<style>
body {
    background-color: #f6f6ef;
    color: #222222;
    font-family: sans-serif;
    margin: 0;
    padding: 8px 12px;
}
.hn-header {
    background-color: #ff6600;
    padding: 4px 8px;
    font-size: 13px;
    font-weight: bold;
    color: #222222;
    margin-bottom: 12px;
}
.hn-title {
    display: inline-block;
    margin-right: 12px;
}
.hn-nav {
    display: inline-block;
    font-size: 12px;
    color: #222222;
}
.hn-item {
    margin-bottom: 10px;
    font-size: 13px;
}
.hn-rank {
    color: #828282;
    display: inline-block;
    width: 24px;
    text-align: right;
    margin-right: 6px;
}
.hn-link {
    color: #000000;
    font-weight: bold;
}
.hn-sub {
    font-size: 10px;
    color: #828282;
    margin-left: 34px;
    margin-top: 2px;
}
</style>
</head>
<body>
<div class=\"hn-header\">
    <div class=\"hn-title\"><b>Y Hacker News</b></div>
    <div class=\"hn-nav\">new | past | comments | ask | show | jobs | submit</div>
</div>

<div class=\"hn-item\">
    <div class=\"hn-rank\">1.</div>
    <span class=\"hn-link\">NovaOS 2.0: Writing an enterprise OS kernel in 100% pure Rust</span>
    <div class=\"hn-sub\">348 points by firoze 2 hours ago | 84 comments</div>
</div>

<div class=\"hn-item\">
    <div class=\"hn-rank\">2.</div>
    <span class=\"hn-link\">Aero: A zero-Chromium pure-Rust web engine with built-in privacy shields</span>
    <div class=\"hn-sub\">295 points by sysrust 3 hours ago | 112 comments</div>
</div>

<div class=\"hn-item\">
    <div class=\"hn-rank\">3.</div>
    <span class=\"hn-link\">Show HN: NovaFS - Copy-on-Write B-Tree enterprise filesystem</span>
    <div class=\"hn-sub\">192 points by kernellab 4 hours ago | 45 comments</div>
</div>

<div class=\"hn-item\">
    <div class=\"hn-rank\">4.</div>
    <span class=\"hn-link\">Why modern operating systems must eliminate C memory unsafety</span>
    <div class=\"hn-sub\">167 points by memory_safe 5 hours ago | 78 comments</div>
</div>

<div class=\"hn-item\">
    <div class=\"hn-rank\">5.</div>
    <span class=\"hn-link\">Building lightweight X11 desktop applications without GTK or Qt</span>
    <div class=\"hn-sub\">134 points by guicraft 6 hours ago | 29 comments</div>
</div>
</body>
</html>".to_owned()
    }

    /// Generates connection error page.
    pub fn error_page(url: &str, error: &str) -> String {
        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>Unable to Connect - Aero</title>
<style>
body {{ background-color: #0b0f19; color: #f1f5f9; padding: 40px 20px; font-family: sans-serif; text-align: center; }}
.card {{ background-color: #1e293b; border: 1px solid #334155; border-radius: 12px; padding: 28px; max-width: 600px; margin: 0 auto; }}
h1 {{ color: #ef4444; font-size: 24px; margin-bottom: 12px; }}
p {{ color: #94a3b8; font-size: 14px; line-height: 1.5; }}
.url-tag {{ background-color: #0f172a; padding: 4px 10px; border-radius: 4px; color: #38bdf8; font-family: monospace; }}
.btn {{ background-color: #3b82f6; color: #fff; padding: 8px 18px; border-radius: 6px; display: inline-block; margin-top: 16px; font-weight: bold; }}
</style>
</head>
<body>
<div class=\"card\">
    <h1>Unable to Connect to Server</h1>
    <p>Aero Browser could not establish a connection to <span class=\"url-tag\">{}</span>.</p>
    <p>{}</p>
    <p>The server may be temporarily unavailable, or your computer's internet connection may be offline.</p>
    <div class=\"btn\">Try Again</div>
</div>
</body>
</html>",
            url, error
        )
    }

    /// Generates the Brave-inspired Shields Control Panel page.
    pub fn shields_page(stats: &ShieldStats) -> String {
        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>Aero Shields</title>
<style>
body {{
    background-color: #0f172a;
    color: #f1f5f9;
    font-family: sans-serif;
    padding: 32px;
}}
.container {{
    max-width: 640px;
    margin: 0 auto;
}}
h1 {{
    color: #ff5500;
    margin-bottom: 8px;
}}
.card {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 20px;
    margin-top: 16px;
}}
.status-badge {{
    background-color: #22c55e;
    color: #ffffff;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 12px;
    display: inline-block;
    margin-bottom: 12px;
}}
</style>
</head>
<body>
<div class=\"container\">
    <h1>Aero Privacy Shields</h1>
    <div class=\"status-badge\">SHIELDS ARE UP</div>
    <p>Aero blocks trackers, advertisements, and malicious scripts automatically without external extensions.</p>

    <div class=\"card\">
        <h3>Protection Statistics</h3>
        <p>Trackers Blocked: <b>{}</b></p>
        <p>HTTPS Upgrades: <b>{}</b></p>
        <p>Scripts Blocked: <b>{}</b></p>
        <p>Bandwidth Saved: <b>{}</b></p>
        <p>Time Saved: <b>{}</b></p>
    </div>
</div>
</body>
</html>",
            stats.trackers_blocked,
            stats.https_upgrades,
            stats.scripts_blocked,
            stats.bandwidth_saved_string(),
            stats.time_saved_string()
        )
    }

    /// Generates the Aero version and architecture spec page.
    pub fn version_page() -> String {
        "<!DOCTYPE html>
<html>
<head>
<title>About Aero</title>
<style>
body {
    background-color: #0f172a;
    color: #f1f5f9;
    font-family: sans-serif;
    padding: 32px;
}
.container {
    max-width: 640px;
    margin: 0 auto;
}
h1 {
    color: #38bdf8;
}
.card {
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 20px;
    margin-top: 16px;
}
</style>
</head>
<body>
<div class=\"container\">
    <h1>Aero Browser v1.0.0</h1>
    <p>Default Modern Pure-Rust Web Browser for NovaOS</p>
    <div class=\"card\">
        <p><b>Engine:</b> AeroEngine 1.0 (Pure Rust DOM, CSS3, & Layout)</p>
        <p><b>Privacy Architecture:</b> Built-in Aero Shields (Brave-inspired)</p>
        <p><b>UI Framework:</b> Multi-Tab Hybrid (Chrome + Firefox Reader)</p>
        <p><b>Host Platform:</b> NovaOS Enterprise (x86 & x86_64 Long Mode)</p>
        <p><b>Memory Safety:</b> 100% Safe Pure Rust (Zero Chromium / Zero WebKit)</p>
    </div>
</div>
</body>
</html>".to_owned()
    }
}
