//! # AeroEngine
//!
//! A fast, modular, pure-Rust web rendering engine featuring:
//! - HTML5 DOM tokenizer & tree builder
//! - CSS3 parser, cascade, and selector specificity resolution
//! - CSS Box Model and Layout reflow engine
//! - Hardware-independent display list builder & software rasterizer
//! - Brave-inspired Privacy & Security Shields (ad/tracker blocker, HTTPS upgrade)
//! - Firefox-inspired Reader Mode (clean readability extraction)
//! - URL parsing & protocol dispatch

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
    borrow::ToOwned,
    format,
};

pub mod css;
pub mod dom;
pub mod font;
pub mod html;
pub mod layout;
pub mod net;
pub mod reader;
pub mod render;
pub mod shields;
pub mod style;

pub use css::{Color, CssParser, StyleSheet, Value};
pub use dom::{ElementData, Node, NodeType};
pub use html::HtmlParser;
pub use layout::{build_layout_tree, Dimensions, EdgeSizes, LayoutBox, Rect};
pub use net::{InternalPages, Url};
pub use reader::{ArticleContent, ReaderEngine, ReaderTheme};
pub use render::{build_display_list, DisplayItem, DisplayList};
pub use shields::{ShieldConfig, ShieldEngine, ShieldStats, UrlAction};
pub use style::{build_style_tree, StyledNode};

/// High-level convenience function to parse HTML and CSS, reflow layout,
/// and rasterize directly onto a 32-bit pixel surface.
pub fn render_html_to_surface(
    html_src: &str,
    surface: &mut [u32],
    viewport_w: usize,
    viewport_h: usize,
    scroll_y: f32,
) -> (f32, DisplayList) {
    // 1. Parse HTML into DOM tree
    let dom_root = HtmlParser::parse(html_src);

    // 2. Load default user-agent stylesheet
    let stylesheet = css::default_user_agent_stylesheet();

    // 3. Resolve CSS styling cascade to build StyledNode tree
    let styled_root = build_style_tree(&dom_root, &stylesheet, None);

    // 4. Construct Layout Box tree
    let mut layout_root = build_layout_tree(&styled_root);

    // 5. Compute layout dimensions
    let mut viewport_dims = Dimensions::default();
    viewport_dims.content.width = viewport_w as f32;
    layout_root.layout(viewport_dims);

    let total_content_height = layout_root.dimensions.content.height;

    // 6. Generate Display List
    let display_list = build_display_list(&layout_root);

    // 7. Rasterize Display List to surface
    display_list.paint(surface, viewport_w, viewport_h, scroll_y);

    (total_content_height, display_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_dom_parsing() {
        let html = r#"
            <div id="main" class="container dark">
                <h1>Welcome to Aero Browser</h1>
                <p>Fast, pure-Rust web engine.</p>
                <a href="https://novaos.org">NovaOS</a>
                <input type="text" value="search" />
                <button>Click Me</button>
            </div>
        "#;
        let doc = HtmlParser::parse(html);
        assert_eq!(doc.node_type, NodeType::Document);

        let main_div = doc.get_element_by_id("main").expect("main div found");
        assert!(main_div.element_data().unwrap().has_class("container"));
        assert!(main_div.element_data().unwrap().has_class("dark"));

        let headings = doc.get_elements_by_tag_name("h1");
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text_content(), "Welcome to Aero Browser");

        let link = doc.get_elements_by_tag_name("a")[0];
        assert_eq!(link.element_data().unwrap().href(), Some("https://novaos.org"));
    }

    #[test]
    fn test_css_parsing_and_specificity() {
        let css = r#"
            h1 { color: #ff5500; font-size: 32px; }
            .container { padding: 16px; background-color: #1e293b; }
            #main { margin: 10px; }
        "#;
        let sheet = CssParser::parse(css);
        assert_eq!(sheet.rules.len(), 3);

        let id_rule = &sheet.rules[2];
        assert_eq!(id_rule.selectors[0].specificity(), (1, 0, 0));

        let class_rule = &sheet.rules[1];
        assert_eq!(class_rule.selectors[0].specificity(), (0, 1, 0));

        let tag_rule = &sheet.rules[0];
        assert_eq!(tag_rule.selectors[0].specificity(), (0, 0, 1));
    }

    #[test]
    fn test_brave_shields_blocking() {
        let mut shields = ShieldEngine::new();
        // Allow clean site
        assert_eq!(
            shields.evaluate_url("https://novaos.org/docs", "novaos.org"),
            UrlAction::Allow("https://novaos.org/docs".to_string())
        );

        // Block tracker
        match shields.evaluate_url("https://analytics.example.org/tracker.js", "example.com") {
            UrlAction::Block { reason } => assert!(reason.contains("analytics.") || reason.contains("tracker.")),
            _ => panic!("Tracker was not blocked!"),
        }
        assert_eq!(shields.stats.trackers_blocked, 1);
        assert!(shields.stats.bandwidth_saved_bytes > 0);

        // Upgrade HTTP to HTTPS
        assert_eq!(
            shields.evaluate_url("http://insecure-site.org/index.html", "insecure-site.org"),
            UrlAction::UpgradeHttps("https://insecure-site.org/index.html".to_string())
        );
        assert_eq!(shields.stats.https_upgrades, 1);
    }

    #[test]
    fn test_reader_mode_extraction() {
        let html = r#"
            <html>
            <head><title>Test Article</title></head>
            <body>
                <nav><a href="/">Home</a><a href="/about">About</a></nav>
                <div class="sidebar">Ads here</div>
                <article>
                    <h1>Aero Engine Architecture</h1>
                    <p>Aero is an independent, pure-Rust layout engine built for modern operating systems.</p>
                    <p>It eliminates dependency on massive Chromium and WebKit runtimes while providing fast page rendering.</p>
                </article>
                <footer>Copyright 2026</footer>
            </body>
            </html>
        "#;
        let doc = HtmlParser::parse(html);
        let article = ReaderEngine::extract(&doc);
        assert_eq!(article.title, "Aero Engine Architecture");
        assert_eq!(article.paragraphs.len(), 2);
        assert!(article.paragraphs[0].contains("pure-Rust layout engine"));

        let reader_html = article.to_reader_html(ReaderTheme::Sepia, 18);
        assert!(reader_html.contains("#f4ecd8")); // Sepia background
        assert!(reader_html.contains("Aero Engine Architecture"));
    }

    #[test]
    fn test_full_surface_rendering() {
        let html = r#"
            <div style="background-color: #ff5500; padding: 10px;">
                <h1>Aero Web</h1>
                <p>Rendering test line.</p>
            </div>
        "#;
        let mut surface = vec![0u32; 800 * 600];
        let (height, display_list) = render_html_to_surface(html, &mut surface, 800, 600, 0.0);
        assert!(height > 0.0);
        assert!(!display_list.items.is_empty());

        // Verify non-zero pixels were drawn onto the surface
        let non_zero = surface.iter().filter(|&&p| p != 0).count();
        assert!(non_zero > 100);
    }
}
