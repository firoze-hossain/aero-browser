//! # Aero Browser Standalone Application
//!
//! Entry point for Aero Browser on host OS and desktop environments.

use aero_ui::AeroBrowser;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let initial_url = args.get(1).map(|s| s.as_str()).unwrap_or("about:newtab");

    println!("==================================================");
    println!("  Aero Browser v1.0.0 (Pure Rust Engine)");
    println!("  Privacy First • Brave Shields • Zero WebKit/Blink");
    println!("==================================================");

    let mut browser = AeroBrowser::new();
    if initial_url != "about:newtab" {
        println!("Navigating to: {}", initial_url);
        browser.navigate(initial_url);
    }

    println!("[OK] Browser initialized successfully.");
    println!("     Active Tab: {}", browser.active_tab().title);
    println!("     Active URL: {}", browser.active_tab().url);
    println!("     Shields: Active (Trackers Blocked: {})", browser.shields.stats.trackers_blocked);
    println!("     Bookmarks: {} saved shortcuts", browser.bookmarks.items.len());

    // Perform an offscreen frame render test (800x600)
    let width = 800;
    let height = 600;
    let mut surface = vec![0u32; width * height];
    browser.render(&mut surface, width, height);

    let drawn_pixels = surface.iter().filter(|&&p| p != 0).count();
    println!("[OK] Frame rasterization complete: {} non-zero pixels rendered.", drawn_pixels);
    println!("==================================================");
}
