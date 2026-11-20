//! # Aero Bookmarks Manager
//!
//! Stores and manages hierarchical bookmarks, folders, and quick-dial shortcuts.
//! Provides interactive folder navigation and instant bookmarking matching Brave and Chrome.

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

/// A single bookmark link or container folder node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookmarkItem {
    Link {
        title: String,
        url: String,
        icon: String,
    },
    Folder {
        title: String,
        children: Vec<BookmarkItem>,
    },
}

impl BookmarkItem {
    /// Returns the display title for this bookmark or folder.
    pub fn title(&self) -> &str {
        match self {
            BookmarkItem::Link { title, .. } => title,
            BookmarkItem::Folder { title, .. } => title,
        }
    }

    /// Returns true if this node is a bookmark folder.
    pub fn is_folder(&self) -> bool {
        matches!(self, BookmarkItem::Folder { .. })
    }

    /// Returns the target URL if this is a bookmark link.
    pub fn url(&self) -> Option<&str> {
        match self {
            BookmarkItem::Link { url, .. } => Some(url),
            BookmarkItem::Folder { .. } => None,
        }
    }

    /// Returns the children if this is a folder.
    pub fn children(&self) -> Option<&[BookmarkItem]> {
        match self {
            BookmarkItem::Folder { children, .. } => Some(children),
            BookmarkItem::Link { .. } => None,
        }
    }
}

/// Backward compatibility alias for single bookmark links.
pub type Bookmark = BookmarkItem;

/// Bookmarks hierarchical storage collection.
#[derive(Debug, Clone)]
pub struct BookmarkManager {
    pub items: Vec<BookmarkItem>,
}

impl BookmarkManager {
    /// Initializes default bookmarks matching user's customized Brave bookmarks bar.
    pub fn new() -> Self {
        BookmarkManager {
            items: vec![
                BookmarkItem::Folder {
                    title: "Vps".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "NovaOS VPS Control".to_owned(),
                            url: "https://vps.novaos.org".to_owned(),
                            icon: "🖥".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "Terminal SSH Console".to_owned(),
                            url: "https://ssh.novaos.org".to_owned(),
                            icon: ">_".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Updated Cover...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "NovaOS Cover Mockup".to_owned(),
                            url: "https://canva.com/novaos-cover".to_owned(),
                            icon: "🎨".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "Aero Branding Assets".to_owned(),
                            url: "https://assets.novaos.org/branding".to_owned(),
                            icon: "🖼".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Dev Tools".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "Rust Documentation".to_owned(),
                            url: "https://doc.rust-lang.org".to_owned(),
                            icon: "⚙".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "Crates.io Registry".to_owned(),
                            url: "https://crates.io".to_owned(),
                            icon: "📦".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Md. Firoze Hos...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "GitHub - firoze-hossain".to_owned(),
                            url: "https://github.com/firoze-hossain".to_owned(),
                            icon: "🐙".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "Personal Portfolio".to_owned(),
                            url: "https://firoze.dev".to_owned(),
                            icon: "👤".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Md. Firoze Hos...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "LinkedIn Profile".to_owned(),
                            url: "https://linkedin.com/in/firoze-hossain".to_owned(),
                            icon: "💼".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Dhul Qarnayn'...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "Dhul Qarnayn Historical Series".to_owned(),
                            url: "https://youtube.com/results?search_query=Dhul+Qarnayn".to_owned(),
                            icon: "▶".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "NAKBangla".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "NAK Bangla Official Channel".to_owned(),
                            url: "https://youtube.com/@nakbangla".to_owned(),
                            icon: "▶".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "NexaCommerce".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "NexaCommerce Store Dashboard".to_owned(),
                            url: "https://nexacommerce.io/admin".to_owned(),
                            icon: "🛒".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "BFDM".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "BFDM Management Portal".to_owned(),
                            url: "https://bfdm.org/portal".to_owned(),
                            icon: "📊".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "Meal Calculati...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "Daily Meal Calculator".to_owned(),
                            url: "https://mealcalc.novaos.org".to_owned(),
                            icon: "🍽".to_owned(),
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "7th Dream Pr...".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "https://drive.google.com/drive/folders/1vBoC...".to_owned(),
                            url: "https://drive.google.com/drive/folders/1vBoC7thDreamProject".to_owned(),
                            icon: "📁".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "NovaOS_Build_Plan.xlsx - Google Sheets".to_owned(),
                            url: "https://docs.google.com/spreadsheets/d/novaos_build_plan".to_owned(),
                            icon: "📊".to_owned(),
                        },
                        BookmarkItem::Folder {
                            title: "novaos".to_owned(),
                            children: vec![
                                BookmarkItem::Link {
                                    title: "Kernel Architecture".to_owned(),
                                    url: "https://github.com/firoze-hossain/novaos".to_owned(),
                                    icon: "⚙".to_owned(),
                                },
                            ],
                        },
                        BookmarkItem::Folder {
                            title: "roze".to_owned(),
                            children: vec![
                                BookmarkItem::Link {
                                    title: "Roze Compiler Engine".to_owned(),
                                    url: "https://github.com/firoze-hossain/roze".to_owned(),
                                    icon: "🦀".to_owned(),
                                },
                            ],
                        },
                        BookmarkItem::Folder {
                            title: "stratosdb".to_owned(),
                            children: vec![
                                BookmarkItem::Link {
                                    title: "StratosDB Distributed Store".to_owned(),
                                    url: "https://github.com/firoze-hossain/stratosdb".to_owned(),
                                    icon: "🗄".to_owned(),
                                },
                            ],
                        },
                    ],
                },
                BookmarkItem::Folder {
                    title: "All Bookmarks".to_owned(),
                    children: vec![
                        BookmarkItem::Link {
                            title: "Manage Bookmarks (Ctrl+Shift+O)".to_owned(),
                            url: "about:bookmarks".to_owned(),
                            icon: "⭐".to_owned(),
                        },
                        BookmarkItem::Link {
                            title: "Import Bookmarks & Settings...".to_owned(),
                            url: "about:settings".to_owned(),
                            icon: "📥".to_owned(),
                        },
                    ],
                },
            ],
        }
    }

    /// Recursively checks whether a URL is currently bookmarked.
    pub fn is_bookmarked(&self, url: &str) -> bool {
        fn check_list(items: &[BookmarkItem], url: &str) -> bool {
            for item in items {
                match item {
                    BookmarkItem::Link { url: u, .. } => {
                        if u == url {
                            return true;
                        }
                    }
                    BookmarkItem::Folder { children, .. } => {
                        if check_list(children, url) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        check_list(&self.items, url)
    }

    /// Toggles bookmarking state for a URL.
    pub fn toggle_bookmark(&mut self, title: &str, url: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|b| b.url() == Some(url)) {
            self.items.remove(pos);
            false
        } else {
            self.items.push(BookmarkItem::Link {
                title: title.to_owned(),
                url: url.to_owned(),
                icon: "★".to_owned(),
            });
            true
        }
    }
}
