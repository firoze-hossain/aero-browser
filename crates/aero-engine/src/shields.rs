//! # Aero Privacy & Security Shields Engine
//!
//! Inspired by Brave Shields: Built-in pure-Rust ad and tracker blocking,
//! automatic HTTPS upgrading, script gating, fingerprinting protection,
//! and privacy savings telemetry.

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

/// High-performance pattern matchers for tracking and advertising endpoints.
pub const TRACKER_BLOCKLIST: &[&str] = &[
    "analytics.",
    "tracker.",
    "telemetry.",
    "beacon.",
    "adservice.",
    "adserver.",
    "/ads/",
    "/pixel.gif",
    "/tracking",
    "/telemetry",
    "/metrics",
];

/// Cumulative privacy metrics saved by Aero Shields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShieldStats {
    pub trackers_blocked: u64,
    pub https_upgrades: u64,
    pub scripts_blocked: u64,
    pub fingerprints_blocked: u64,
    pub bandwidth_saved_bytes: u64,
    pub time_saved_ms: u64,
}

impl ShieldStats {
    /// Formatted bandwidth saved string (e.g. "24.5 KB", "1.2 MB").
    pub fn bandwidth_saved_string(&self) -> String {
        if self.bandwidth_saved_bytes < 1024 {
            format!("{} B", self.bandwidth_saved_bytes)
        } else if self.bandwidth_saved_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.bandwidth_saved_bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", self.bandwidth_saved_bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// Formatted time saved string (e.g. "450 ms", "1.8 s").
    pub fn time_saved_string(&self) -> String {
        if self.time_saved_ms < 1000 {
            format!("{} ms", self.time_saved_ms)
        } else {
            format!("{:.1} s", self.time_saved_ms as f64 / 1000.0)
        }
    }
}

/// Active settings for the privacy shield.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShieldConfig {
    /// Master toggle for all shields on the active site.
    pub enabled: bool,
    /// Block trackers, third-party analytics, and aggressive ad scripts.
    pub block_trackers_and_ads: bool,
    /// Automatically upgrade unencrypted HTTP connections to HTTPS.
    pub upgrade_https: bool,
    /// Block untrusted JavaScript and third-party script execution.
    pub block_scripts: bool,
    /// Prevent canvas and audio fingerprinting.
    pub prevent_fingerprinting: bool,
    /// Block third-party tracking cookies.
    pub block_cross_site_cookies: bool,
}

impl Default for ShieldConfig {
    fn default() -> Self {
        ShieldConfig {
            enabled: true,
            block_trackers_and_ads: true,
            upgrade_https: true,
            block_scripts: false,
            prevent_fingerprinting: true,
            block_cross_site_cookies: true,
        }
    }
}

/// The Brave-inspired Shield protection engine.
#[derive(Debug, Clone)]
pub struct ShieldEngine {
    pub config: ShieldConfig,
    pub stats: ShieldStats,
    pub site_shields: Vec<(String, bool)>, // (domain, enabled)
}

impl ShieldEngine {
    pub fn new() -> Self {
        ShieldEngine {
            config: ShieldConfig::default(),
            stats: ShieldStats::default(),
            site_shields: Vec::new(),
        }
    }

    /// Checks if shields are active for a specific domain.
    pub fn is_shields_up_for_domain(&self, domain: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        for (d, enabled) in &self.site_shields {
            if d == domain {
                return *enabled;
            }
        }
        true
    }

    /// Toggles shields for a specific site domain.
    pub fn set_shields_for_domain(&mut self, domain: &str, enabled: bool) {
        if let Some(pos) = self.site_shields.iter().position(|(d, _)| d == domain) {
            self.site_shields[pos].1 = enabled;
        } else {
            self.site_shields.push((domain.to_owned(), enabled));
        }
    }

    /// Toggles shields on/off for a domain.
    pub fn toggle_shields_for_domain(&mut self, domain: &str) {
        let current = self.is_shields_up_for_domain(domain);
        self.set_shields_for_domain(domain, !current);
    }

    /// Inspects an outgoing URL request and decides whether to allow, block, or upgrade it.
    pub fn evaluate_url(&mut self, url: &str, current_domain: &str) -> UrlAction {
        if !self.is_shields_up_for_domain(current_domain) {
            return UrlAction::Allow(url.to_owned());
        }

        let url_lower = url.to_lowercase();

        // 1. Check Ad & Tracker Blocklist
        if self.config.block_trackers_and_ads {
            for pattern in TRACKER_BLOCKLIST {
                if url_lower.contains(pattern) {
                    self.stats.trackers_blocked += 1;
                    // Approximate ~45KB and ~60ms saved per blocked tracker script/ad
                    self.stats.bandwidth_saved_bytes += 45 * 1024;
                    self.stats.time_saved_ms += 60;
                    return UrlAction::Block {
                        reason: format!("Blocked tracker/ad matching '{}'", pattern),
                    };
                }
            }
        }

        // 2. Check Script Blocking
        if self.config.block_scripts && (url_lower.ends_with(".js") || url_lower.contains(".js?")) {
            self.stats.scripts_blocked += 1;
            return UrlAction::Block {
                reason: "Script execution blocked by Shield policy".to_owned(),
            };
        }

        // 3. Automatic HTTPS Upgrading
        if self.config.upgrade_https && url_lower.starts_with("http://") {
            let upgraded = format!("https://{}", &url[7..]);
            self.stats.https_upgrades += 1;
            return UrlAction::UpgradeHttps(upgraded);
        }

        UrlAction::Allow(url.to_owned())
    }
}

/// Action determined by the Shields evaluation engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlAction {
    /// Safe URL permitted to load.
    Allow(String),
    /// URL upgraded from insecure HTTP to HTTPS.
    UpgradeHttps(String),
    /// Malicious, tracking, or advertising URL dropped.
    Block { reason: String },
}
