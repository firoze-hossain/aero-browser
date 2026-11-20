//! # Aero Reader Mode (Distraction-Free Reading View)
//!
//! Inspired by Firefox Reader View: Parses DOM trees, scores content density,
//! strips navigation boilerplate, advertisements, and clutter, and generates a
//! clean, beautifully typeset article view with customizable typography and themes.

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

use crate::dom::{Node, NodeType};

/// Visual theme for Reader Mode view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderTheme {
    Light,
    Dark,
    Sepia,
    Paper,
}

impl ReaderTheme {
    pub fn background_color(&self) -> &'static str {
        match self {
            ReaderTheme::Light => "#ffffff",
            ReaderTheme::Dark => "#18181b",
            ReaderTheme::Sepia => "#f4ecd8",
            ReaderTheme::Paper => "#f7f7f7",
        }
    }

    pub fn text_color(&self) -> &'static str {
        match self {
            ReaderTheme::Light => "#1e293b",
            ReaderTheme::Dark => "#f1f5f9",
            ReaderTheme::Sepia => "#5b4636",
            ReaderTheme::Paper => "#222222",
        }
    }

    pub fn card_bg(&self) -> &'static str {
        match self {
            ReaderTheme::Light => "#f8fafc",
            ReaderTheme::Dark => "#27272a",
            ReaderTheme::Sepia => "#eee2cb",
            ReaderTheme::Paper => "#ffffff",
        }
    }
}

/// Extracted article content representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ArticleContent {
    pub title: String,
    pub byline: Option<String>,
    pub paragraphs: Vec<String>,
    pub reading_time_minutes: usize,
}

impl ArticleContent {
    /// Converts the extracted article into a clean, standalone HTML page ready for AeroEngine.
    pub fn to_reader_html(&self, theme: ReaderTheme, font_size_px: usize) -> String {
        let mut paragraphs_html = String::new();
        for p in &self.paragraphs {
            paragraphs_html.push_str(&format!("<p>{}</p>\n", p));
        }

        let byline_html = self
            .byline
            .as_ref()
            .map(|b| format!("<div class=\"byline\">{} • {} min read</div>", b, self.reading_time_minutes))
            .unwrap_or_else(|| format!("<div class=\"byline\">{} min read</div>", self.reading_time_minutes));

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - Aero Reader</title>
<style>
body {{
    background-color: {};
    color: {};
    font-size: {}px;
    line-height: 1.8;
    margin: 0;
    padding: 32px 24px;
}}
.reader-container {{
    max-width: 680px;
    margin: 0 auto;
}}
h1 {{
    font-size: 32px;
    margin-bottom: 8px;
    line-height: 1.3;
}}
.byline {{
    font-size: 14px;
    color: #94a3b8;
    margin-bottom: 24px;
    border-bottom: 1px solid #334155;
    padding-bottom: 12px;
}}
p {{
    margin-bottom: 18px;
    text-align: justify;
}}
</style>
</head>
<body>
<div class=\"reader-container\">
    <h1>{}</h1>
    {}
    {}
</div>
</body>
</html>",
            self.title,
            theme.background_color(),
            theme.text_color(),
            font_size_px,
            self.title,
            byline_html,
            paragraphs_html
        )
    }
}

/// Reader mode content extractor engine.
pub struct ReaderEngine;

impl ReaderEngine {
    /// Extracts the main article title and body paragraphs from an arbitrary DOM tree.
    pub fn extract(root: &Node) -> ArticleContent {
        let mut title = "Untitled Document".to_owned();
        if let Some(h1) = root.get_elements_by_tag_name("h1").first() {
            let t = h1.text_content().trim().to_owned();
            if !t.is_empty() {
                title = t;
            }
        } else if let Some(title_elem) = root.get_elements_by_tag_name("title").first() {
            let t = title_elem.text_content().trim().to_owned();
            if !t.is_empty() {
                title = t;
            }
        }

        let mut paragraphs = Vec::new();
        Self::collect_article_paragraphs(root, &mut paragraphs);

        // If no paragraphs found, extract from general text
        if paragraphs.is_empty() {
            let full_text = root.text_content();
            for block in full_text.split("\n\n") {
                let trimmed = block.trim();
                if trimmed.len() > 20 {
                    paragraphs.push(trimmed.to_owned());
                }
            }
        }

        let word_count: usize = paragraphs
            .iter()
            .map(|p| p.split_whitespace().count())
            .sum();
        let reading_time_minutes = (word_count / 200).max(1);

        ArticleContent {
            title,
            byline: Some("Aero Reader Mode".to_owned()),
            paragraphs,
            reading_time_minutes,
        }
    }

    fn collect_article_paragraphs(node: &Node, out: &mut Vec<String>) {
        if let Some(tag) = node.tag_name() {
            // Skip non-content tags
            match tag {
                "nav" | "header" | "footer" | "aside" | "script" | "style" | "button" => return,
                _ => {}
            }

            if tag == "p" || tag == "blockquote" {
                let text = node.text_content().trim().to_owned();
                // Filter out tiny navigation links/buttons mistakenly wrapped in <p>
                if text.len() > 25 {
                    out.push(text);
                }
                return;
            }
        }

        for child in &node.children {
            Self::collect_article_paragraphs(child, out);
        }
    }
}
