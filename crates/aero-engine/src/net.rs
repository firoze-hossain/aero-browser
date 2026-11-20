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
    /// Generates the Brave-inspired celestial New Tab Dashboard page.
    pub fn new_tab(stats: &ShieldStats) -> String {
        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>New Tab</title>
<style>
body {{
    background-color: #0b0f19;
    color: #f1f5f9;
    font-family: sans-serif;
    margin: 0;
    padding: 36px 20px;
    text-align: center;
}}
.dashboard {{
    max-width: 820px;
    margin: 0 auto;
}}
.logo-title {{
    font-size: 28px;
    font-weight: bold;
    color: #ff5500;
    margin-bottom: 8px;
}}
.subtitle {{
    font-size: 14px;
    color: #94a3b8;
    margin-bottom: 26px;
}}
.stats-bar {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 12px;
    padding: 16px 20px;
    margin-bottom: 28px;
    display: block;
}}
.stat-item {{
    display: inline-block;
    width: 220px;
    margin: 4px 10px;
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
.search-box {{
    background-color: #1e293b;
    border: 1px solid #475569;
    border-radius: 20px;
    padding: 12px 20px;
    width: 60%;
    margin: 0 auto 28px auto;
    color: #94a3b8;
    font-size: 14px;
}}
.quick-dial {{
    margin-top: 16px;
}}
.tile {{
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 10px 18px;
    display: inline-block;
    margin: 6px 8px;
    color: #f8fafc;
    font-size: 13px;
}}
</style>
</head>
<body>
<div class=\"dashboard\">
    <div class=\"logo-title\">AERO BROWSER</div>
    <div class=\"subtitle\">Fast, Private, Pure-Rust Web Navigator for NovaOS</div>

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

    <div class=\"search-box\">Search the web or type a URL...</div>

    <div class=\"quick-dial\">
        <div class=\"tile\">NovaOS Hub</div>
        <div class=\"tile\">Rust Programming</div>
        <div class=\"tile\">GitHub</div>
        <div class=\"tile\">Wikipedia</div>
        <div class=\"tile\">Hacker News</div>
        <div class=\"tile\">DuckDuckGo</div>
    </div>
</div>
</body>
</html>",
            stats.trackers_blocked,
            stats.bandwidth_saved_string(),
            stats.time_saved_string()
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
