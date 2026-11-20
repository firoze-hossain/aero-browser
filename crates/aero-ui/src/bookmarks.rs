//! # Aero Bookmarks Manager
//!
//! Stores and manages favorite bookmarks and quick-dial shortcuts.

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

/// A single saved bookmark entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
    pub icon: &'static str,
}

/// Bookmarks storage collection.
#[derive(Debug, Clone)]
pub struct BookmarkManager {
    pub items: Vec<Bookmark>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        BookmarkManager {
            items: vec![
                Bookmark {
                    title: "NovaOS Portal".to_owned(),
                    url: "https://novaos.org".to_owned(),
                    icon: "★",
                },
                Bookmark {
                    title: "Rust Lang".to_owned(),
                    url: "https://rust-lang.org".to_owned(),
                    icon: "⚙",
                },
                Bookmark {
                    title: "GitHub".to_owned(),
                    url: "https://github.com".to_owned(),
                    icon: "🐙",
                },
                Bookmark {
                    title: "Wikipedia".to_owned(),
                    url: "https://wikipedia.org".to_owned(),
                    icon: "W",
                },
                Bookmark {
                    title: "Hacker News".to_owned(),
                    url: "https://news.ycombinator.com".to_owned(),
                    icon: "Y",
                },
                Bookmark {
                    title: "Aero Shields".to_owned(),
                    url: "about:shields".to_owned(),
                    icon: "🛡",
                },
            ],
        }
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.items.iter().any(|b| b.url == url)
    }

    pub fn toggle_bookmark(&mut self, title: &str, url: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|b| b.url == url) {
            self.items.remove(pos);
            false
        } else {
            self.items.push(Bookmark {
                title: title.to_owned(),
                url: url.to_owned(),
                icon: "★",
            });
            true
        }
    }
}
