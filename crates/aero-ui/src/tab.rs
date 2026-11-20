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

/// Video metadata representation for authentic YouTube experience.
#[derive(Debug, Clone)]
pub struct YouTubeVideo {
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub views: String,
    pub duration: String,
    pub published: String,
    pub description: String,
    pub verified: bool,
}

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
    pub fn get_default_home_feed() -> Vec<YouTubeVideo> {
        vec![
            YouTubeVideo {
                video_id: "yt_noman2".to_owned(),
                title: "Apni Allahar Hoye Jan (Turn to Allah) || Shaykh Nouman Ali Khan".to_owned(),
                channel: "Shaykh Nouman Ali Khan Bangla".to_owned(),
                views: "40K views".to_owned(),
                duration: "21:15".to_owned(),
                published: "3d ago".to_owned(),
                description: "Spiritual lecture on seeking closeness to Allah and worldly peace.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "yt_parliament".to_owned(),
                title: "Mugdho Puro Sangsad (Historic Parliament Speech) | ATN News".to_owned(),
                channel: "ATN News".to_owned(),
                views: "1.2M views".to_owned(),
                duration: "22:34".to_owned(),
                published: "2d ago".to_owned(),
                description: "Historic parliament address sparking major national headlines.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "yt_humble".to_owned(),
                title: "True Men of Knowledge are Extremely Humble || Dr. Noman Ali Khan".to_owned(),
                channel: "Motivation".to_owned(),
                views: "6.5K views".to_owned(),
                duration: "17:06".to_owned(),
                published: "2d ago".to_owned(),
                description: "In-depth lecture on humility, wisdom, and character of scholars.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "yt_barsha".to_owned(),
                title: "Barita Aji Chere Dite Hobe (Eviction Report) | Ananta Jalil | ATN News".to_owned(),
                channel: "ATN News".to_owned(),
                views: "106K views".to_owned(),
                duration: "1:14".to_owned(),
                published: "23h ago".to_owned(),
                description: "Latest entertainment news report from national newsroom.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "yt_mashrafe".to_owned(),
                title: "Dirghodin Por Dekha Millo Mashrafe (Mashrafe Exclusive) | Khabar Protidin".to_owned(),
                channel: "Khabar Protidin".to_owned(),
                views: "2.7M views".to_owned(),
                duration: "28:15".to_owned(),
                published: "2d ago".to_owned(),
                description: "Special interview with former cricket captain Mashrafe Bin Mortaza.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "4NlKj7Saogc".to_owned(),
                title: "Muslim Torunder Uddesshe Kichu Kotha || Nouman Ali Khan (NAKInBangla)".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "155K views".to_owned(),
                duration: "56:28".to_owned(),
                published: "1y ago".to_owned(),
                description: "Support us in patreon: https://patreon.com/spektrummedia Advice for Muslim Youth.".to_owned(),
                verified: true,
            },
        ]
    }

    #[cfg(feature = "std")]
    pub fn get_default_search_results(query: &str) -> Vec<YouTubeVideo> {
        let q_lower = query.to_lowercase();
        if q_lower.contains("vlad") || q_lower.contains("kids") {
            vec![
                Self::get_video_by_id("bY1cozLiK14"),
                Self::get_video_by_id("4NlKj7Saogc"),
                Self::get_video_by_id("pagi23"),
            ]
        } else if q_lower.contains("rust") || q_lower.contains("novaos") || q_lower.contains("program") {
            vec![
                Self::get_video_by_id("XsNGS_bxzwg"),
                Self::get_video_by_id("4NlKj7Saogc"),
                Self::get_video_by_id("yt_noman2"),
            ]
        } else {
            vec![
                Self::get_video_by_id("bY1cozLiK14"),
                Self::get_video_by_id("4NlKj7Saogc"),
                Self::get_video_by_id("pagi23"),
                Self::get_video_by_id("LWzPB04DhCg"),
                Self::get_video_by_id("XsNGS_bxzwg"),
            ]
        }
    }

    #[cfg(feature = "std")]
    pub fn get_video_by_id(vid: &str) -> YouTubeVideo {
        match vid {
            "bY1cozLiK14" => YouTubeVideo {
                video_id: "bY1cozLiK14".to_owned(),
                title: "Vlad and Niki new mother explore ice cream truck (ভ্লাদ এবং নিকি)".to_owned(),
                channel: "Niki TV".to_owned(),
                views: "11M views".to_owned(),
                duration: "4:47".to_owned(),
                published: "5y ago".to_owned(),
                description: "Vlad and Niki explore new ice cream truck adventure for kids with giant gelato cones!".to_owned(),
                verified: true,
            },
            "4NlKj7Saogc" => YouTubeVideo {
                video_id: "4NlKj7Saogc".to_owned(),
                title: "Muslim Torunder Uddesshe Kichu Kotha || Nouman Ali Khan (NAKInBangla)".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "155K views".to_owned(),
                duration: "56:31".to_owned(),
                published: "1y ago".to_owned(),
                description: "Advice and spiritual guidance for Muslim youth facing modern challenges by Dr. Nouman Ali Khan.".to_owned(),
                verified: true,
            },
            "pagi23" => YouTubeVideo {
                video_id: "pagi23".to_owned(),
                title: "Pagol Naki Song (Original) | পাগল নাকি | WAQAS (Official Lyric Video)".to_owned(),
                channel: "WAQAS".to_owned(),
                views: "348K views".to_owned(),
                duration: "2:38".to_owned(),
                published: "2w ago".to_owned(),
                description: "Pagol Naki official acoustic studio session | Bengali Diaries Volume 1 by WAQAS.".to_owned(),
                verified: true,
            },
            "LWzPB04DhCg" => YouTubeVideo {
                video_id: "LWzPB04DhCg".to_owned(),
                title: "Vul Naki Prem | ভুল নাকি প্রেম | Arosh Khan | Bangla Natok 2026".to_owned(),
                channel: "G Series Bangla Natok".to_owned(),
                views: "3.9M views".to_owned(),
                duration: "44:59".to_owned(),
                published: "6mo ago".to_owned(),
                description: "Special Eid romantic drama 'Vul Naki Prem' starring Arosh Khan and Tania Bristy.".to_owned(),
                verified: true,
            },
            "XsNGS_bxzwg" => YouTubeVideo {
                video_id: "XsNGS_bxzwg".to_owned(),
                title: "Building NovaOS 2.0: Writing an Operating System in Rust".to_owned(),
                channel: "NovaOS Engineering".to_owned(),
                views: "542K views".to_owned(),
                duration: "42:15".to_owned(),
                published: "3d ago".to_owned(),
                description: "Architecture deep dive into monolithic kernel engineering in 100% pure safe Rust.".to_owned(),
                verified: true,
            },
            "yt_noman2" => YouTubeVideo {
                video_id: "yt_noman2".to_owned(),
                title: "Apni Allahar Hoye Jan (Turn to Allah) || Shaykh Nouman Ali Khan".to_owned(),
                channel: "Shaykh Nouman Ali Khan Bangla".to_owned(),
                views: "40K views".to_owned(),
                duration: "21:15".to_owned(),
                published: "3d ago".to_owned(),
                description: "Spiritual lecture on seeking closeness to Allah and finding inner tranquility in life.".to_owned(),
                verified: true,
            },
            "yt_parliament" => YouTubeVideo {
                video_id: "yt_parliament".to_owned(),
                title: "Mugdho Puro Sangsad (Historic Parliament Speech) | ATN News".to_owned(),
                channel: "ATN News".to_owned(),
                views: "1.2M views".to_owned(),
                duration: "22:34".to_owned(),
                published: "2d ago".to_owned(),
                description: "Historic parliament address sparking major national headlines and public discussion.".to_owned(),
                verified: true,
            },
            "yt_humble" | "rel_humble" => YouTubeVideo {
                video_id: "yt_humble".to_owned(),
                title: "True Men of Knowledge are Extremely Humble || Dr. Noman Ali Khan".to_owned(),
                channel: "Motivation".to_owned(),
                views: "6.5K views".to_owned(),
                duration: "17:06".to_owned(),
                published: "2d ago".to_owned(),
                description: "In-depth lecture exploring humility, wisdom, and character development in true scholars.".to_owned(),
                verified: true,
            },
            "yt_barsha" => YouTubeVideo {
                video_id: "yt_barsha".to_owned(),
                title: "Barita Aji Chere Dite Hobe (Eviction Report) | Ananta Jalil | ATN News".to_owned(),
                channel: "ATN News".to_owned(),
                views: "106K views".to_owned(),
                duration: "1:14".to_owned(),
                published: "23h ago".to_owned(),
                description: "Latest entertainment news investigation from national broadcast newsroom.".to_owned(),
                verified: true,
            },
            "yt_mashrafe" => YouTubeVideo {
                video_id: "yt_mashrafe".to_owned(),
                title: "Dirghodin Por Dekha Millo Mashrafe (Mashrafe Exclusive) | Khabar Protidin".to_owned(),
                channel: "Khabar Protidin".to_owned(),
                views: "2.7M views".to_owned(),
                duration: "28:15".to_owned(),
                published: "2d ago".to_owned(),
                description: "Exclusive interview with former cricket captain Mashrafe Bin Mortaza on sports & life.".to_owned(),
                verified: true,
            },
            "rel_hajj" => YouTubeVideo {
                video_id: "rel_hajj".to_owned(),
                title: "Hojjer Pothe Nobiji Part 4 || Nouman Ali Khan".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "19K views".to_owned(),
                duration: "45:53".to_owned(),
                published: "1y ago".to_owned(),
                description: "The historical and spiritual journey of Hajj and following the footsteps of the Prophet.".to_owned(),
                verified: true,
            },
            "rel_sirah" => YouTubeVideo {
                video_id: "rel_sirah".to_owned(),
                title: "Sobar Jibonei Sukher Obhinoy Keno Eto Nikhut? - Nouman Ali Khan".to_owned(),
                channel: "SiRah".to_owned(),
                views: "1.3K views".to_owned(),
                duration: "38:57".to_owned(),
                published: "1d ago".to_owned(),
                description: "Deep reflections on societal expectations, mental peace, and authentic contentment.".to_owned(),
                verified: true,
            },
            "rel_marriage" => YouTubeVideo {
                video_id: "rel_marriage".to_owned(),
                title: "Biyer Purbe Kichu Proyojonio Poramorsho • Nouman Ali Khan".to_owned(),
                channel: "Bangla Waz".to_owned(),
                views: "304K views".to_owned(),
                duration: "30:32".to_owned(),
                published: "11mo ago".to_owned(),
                description: "Crucial guidance and advice on marriage preparation, mutual respect, and family building.".to_owned(),
                verified: true,
            },
            "rel_destroy" => YouTubeVideo {
                video_id: "rel_destroy".to_owned(),
                title: "Amra Nijekey Dhongsho Kore Felchi? | Noman Ali Khan Bangla".to_owned(),
                channel: "Islam Daily".to_owned(),
                views: "2.3K views".to_owned(),
                duration: "24:10".to_owned(),
                published: "1y ago".to_owned(),
                description: "Are we destroying our own potential? Soul searching lecture by Nouman Ali Khan.".to_owned(),
                verified: true,
            },
            _ => YouTubeVideo {
                video_id: vid.to_owned(),
                title: format!("Special Feature: {}", vid),
                channel: "Aero Media Official".to_owned(),
                views: "100K views".to_owned(),
                duration: "15:00".to_owned(),
                published: "Recently".to_owned(),
                description: "Aero Browser media broadcast.".to_owned(),
                verified: true,
            },
        }
    }

    #[cfg(feature = "std")]
    pub fn get_video_thumbnail_details(v: &YouTubeVideo) -> (
        &'static str, // bg_color
        &'static str, // border_color
        &'static str, // badge_bg
        &'static str, // badge_color
        &'static str, // tag
        &'static str, // headline
        &'static str, // subtext
        &'static str, // avatar_bg
        &'static str, // avatar_text
    ) {
        if v.title.contains("Vlad") || v.channel.contains("Niki") {
            (
                "#0284c7",
                "#38bdf8",
                "#0f172a",
                "#38bdf8",
                "🍦 KIDS FUN ADVENTURE",
                "🌟 ICE CREAM TRUCK QUEST!",
                "Vlad &amp; Niki • 4K Gelato Fun",
                "#0284c7",
                "VLAD",
            )
        } else if v.title.contains("Pagol Naki") || v.channel.contains("WAQAS") {
            (
                "#581c87",
                "#a855f7",
                "#0f172a",
                "#c084fc",
                "🎸 OFFICIAL MUSIC VIDEO",
                "🎵 PAGOL NAKI • STUDIO MASTER",
                "WAQAS Bengali Diaries Vol 1",
                "#a855f7",
                "WAQAS",
            )
        } else if v.title.contains("Vul Naki Prem") || v.channel.contains("G Series") {
            (
                "#7f1d1d",
                "#ef4444",
                "#0f172a",
                "#f87171",
                "🎬 BANGLA NATOK 2026",
                "🎭 VUL NAKI PREM • ROMANCE",
                "Arosh Khan &amp; Tania Bristy",
                "#b91c1c",
                "G",
            )
        } else if v.title.contains("NovaOS") || v.channel.contains("NovaOS") {
            (
                "#0f172a",
                "#3b82f6",
                "#1e3a8a",
                "#60a5fa",
                "⚡ RUST KERNEL DEV",
                "🦀 BUILDING NOVAOS 2.0",
                "Monolithic OS in Pure Rust",
                "#ff5500",
                "NOVA",
            )
        } else if v.title.contains("Parliament") || (v.channel.contains("ATN") && v.title.contains("Mugdho")) {
            (
                "#881337",
                "#f43f5e",
                "#0f172a",
                "#fda4af",
                "📺 NATIONAL PARLIAMENT",
                "📡 MUGDHO PURO SANGSAD",
                "ATN News Prime Time Report",
                "#dc2626",
                "ATN",
            )
        } else if v.title.contains("Mashrafe") {
            (
                "#14532d",
                "#22c55e",
                "#052e16",
                "#86efac",
                "🏏 CRICKET EXCLUSIVE",
                "🏆 MASHRAFE INTERVIEW",
                "Khabar Protidin Sports Special",
                "#16a34a",
                "KP",
            )
        } else if v.title.contains("Barita") {
            (
                "#713f12",
                "#eab308",
                "#1c1917",
                "#fde047",
                "📰 SPECIAL INVESTIGATION",
                "🏠 BARITA AJI CHERE DITE HOBE",
                "Entertainment News Investigation",
                "#dc2626",
                "ATN",
            )
        } else {
            // Default / Islamic / Nouman Ali Khan
            (
                "#064e3b",
                "#059669",
                "#0f172a",
                "#34d399",
                "🕌 BAYYINAH ISLAMIC WISDOM",
                "📖 SPIRITUAL LECTURE • 1080p",
                "Dr. Nouman Ali Khan Official",
                "#cc0000",
                "NAK",
            )
        }
    }

    #[cfg(feature = "std")]
    pub fn get_related_videos_for(video_id: &str) -> Vec<YouTubeVideo> {
        vec![
            Self::get_video_by_id("rel_hajj"),
            Self::get_video_by_id("rel_sirah"),
            Self::get_video_by_id("rel_marriage"),
            Self::get_video_by_id("rel_destroy"),
            Self::get_video_by_id("rel_humble"),
        ]
    }

    #[cfg(feature = "std")]
    fn fetch_live_youtube(query: Option<&str>) -> Option<(String, String)> {
        let endpoint = if let Some(q) = query {
            let encoded = q.replace(' ', "+");
            format!("https://www.youtube.com/results?search_query={}", encoded)
        } else {
            "https://www.youtube.com/results?search_query=trending".to_owned()
        };

        let mut videos = Vec::new();
        if let Ok(resp) = minreq::get(&endpoint)
            .with_timeout(6)
            .with_max_redirects(5)
            .with_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
        {
            if resp.status_code >= 200 && resp.status_code < 400 {
                if let Ok(body) = resp.as_str() {
                    videos = Self::parse_youtube_videos_from_html(body);
                }
            }
        }

        if let Some(q) = query {
            if videos.is_empty() {
                videos = Self::get_default_search_results(q);
            }
            let html = Self::build_youtube_search_results_html(q, &videos, q, false, false);
            Some((format!("{} - YouTube", q), html))
        } else {
            if videos.is_empty() {
                videos = Self::get_default_home_feed();
            }
            let html = Self::build_youtube_feed_html("All", &videos, "", false, false);
            Some(("YouTube".to_owned(), html))
        }
    }

    #[cfg(feature = "std")]
    fn parse_youtube_videos_from_html(html: &str) -> Vec<YouTubeVideo> {
        let mut results = Vec::new();
        let mut cursor = 0;
        while let Some(vr_idx) = html[cursor..].find("\"videoRenderer\":{") {
            let abs_idx = cursor + vr_idx;
            cursor = abs_idx + 17;

            let end_idx = (cursor + 3500).min(html.len());
            let chunk = &html[cursor..end_idx];

            let video_id = if let Some(v_idx) = chunk.find("\"videoId\":\"") {
                let v_start = v_idx + 11;
                if let Some(v_end) = chunk[v_start..].find("\"") {
                    chunk[v_start..v_start + v_end].to_owned()
                } else {
                    continue;
                }
            } else {
                continue;
            };

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

            let published = if let Some(p_idx) = chunk.find("\"publishedTimeText\":{\"simpleText\":\"") {
                let p_start = p_idx + 35;
                if let Some(p_end) = chunk[p_start..].find("\"") {
                    chunk[p_start..p_start + p_end].to_owned()
                } else {
                    "Recent".to_owned()
                }
            } else {
                "Recent".to_owned()
            };

            let description = if let Some(ds_idx) = chunk.find("\"snippetText\":{\"runs\":[{\"text\":\"") {
                let ds_start = ds_idx + 32;
                if let Some(ds_end) = chunk[ds_start..].find("\"") {
                    chunk[ds_start..ds_start + ds_end].to_owned()
                } else {
                    "".to_owned()
                }
            } else {
                "".to_owned()
            };

            results.push(YouTubeVideo {
                video_id,
                title,
                channel,
                views,
                duration,
                published,
                description,
                verified: true,
            });
            if results.len() >= 12 {
                break;
            }
        }
        results
    }

    #[cfg(feature = "std")]
    pub fn parse_duration_to_sec(dur: &str) -> u32 {
        let parts: Vec<&str> = dur.split(':').collect();
        if parts.len() == 2 {
            let m: u32 = parts[0].trim().parse().unwrap_or(5);
            let s: u32 = parts[1].trim().parse().unwrap_or(0);
            m * 60 + s
        } else if parts.len() == 3 {
            let h: u32 = parts[0].trim().parse().unwrap_or(1);
            let m: u32 = parts[1].trim().parse().unwrap_or(0);
            let s: u32 = parts[2].trim().parse().unwrap_or(0);
            h * 3600 + m * 60 + s
        } else {
            300
        }
    }

    #[cfg(feature = "std")]
    pub fn get_player_stage_details(v: &YouTubeVideo) -> (
        &'static str, // hud_badge
        &'static str, // slide_tag
        String,       // slide_main
        [&'static str; 3], // bullets
        String,       // speaker_title
        String,       // speaker_sub
        String,       // cc_subtitle
        &'static str, // avatar_bg
        &'static str, // avatar_text
    ) {
        if v.title.contains("Vlad") || v.channel.contains("Niki") {
            (
                "● LIVE 4K UHD 60FPS • Niki TV Official Stream",
                "🍦 KIDS ADVENTURE &amp; ENTERTAINMENT",
                "Vlad &amp; Niki: Ice Cream Truck Expedition".to_owned(),
                [
                    "• Exploring the magical mega ice cream truck",
                    "• Flavors quest: Strawberry, Gelato &amp; Bubblegum",
                    "• Outdoor backyard fun with brother Niki &amp; mom",
                ],
                "[ 🌟 VLAD &amp; NIKI ]".to_owned(),
                "Kids Creators • 110M+ Global Subscribers".to_owned(),
                "&quot;Look at this giant gelato cone! We found all five rainbow flavors!&quot;".to_owned(),
                "#0284c7",
                "VLAD",
            )
        } else if v.title.contains("Pagol Naki") || v.channel.contains("WAQAS") {
            (
                "● LIVE 1080p HI-FI • Studio Acoustic Session",
                "🎸 OFFICIAL ACOUSTIC MASTER",
                "Pagol Naki (Original) • WAQAS Acoustic Live".to_owned(),
                [
                    "• Bengali Diaries Vol 1: Studio Sound Engineering",
                    "• Unplugged guitar &amp; intimate acoustic vocals",
                    "• Written, composed and performed by WAQAS",
                ],
                "[ 🎵 WAQAS ]".to_owned(),
                "Singer-Songwriter • Studio Sessions".to_owned(),
                "&quot;Tumi ki pagol naki... amar moner kotha shune jao ekbar...&quot;".to_owned(),
                "#a855f7",
                "WAQAS",
            )
        } else if v.title.contains("Vul Naki Prem") || v.channel.contains("G Series") {
            (
                "● LIVE 1080p60 FHD • G Series Official Broadcast",
                "🎬 BANGLA NATOK 2026 • SPECIAL DRAMA",
                "Vul Naki Prem • Arosh Khan &amp; Tania Bristy".to_owned(),
                [
                    "• Directed by G Series Special Drama Productions",
                    "• Romantic comedy drama set across old Dhaka",
                    "• Original soundtrack by Autumnal Moon",
                ],
                "[ 🎭 AROSH KHAN ]".to_owned(),
                "Lead Actor • G Series Bangla Drama".to_owned(),
                "&quot;Ei shohore shobai bhalobasha khuje, kintu shotti prem ki emni pawa jaay?&quot;".to_owned(),
                "#ef4444",
                "G",
            )
        } else if v.title.contains("NovaOS") || v.channel.contains("NovaOS") {
            (
                "● LIVE 1080p60 HD • Pure Rust Systems Architecture",
                "⚡ RUST KERNEL DEVELOPMENT",
                "Building NovaOS 2.0: Writing an OS in Pure Rust".to_owned(),
                [
                    "• Monolithic zero-dependency x86_64 architecture",
                    "• nova_uring async I/O rings &amp; memory safety",
                    "• Pure Rust Aero Browser rendering engine",
                ],
                "[ 🦀 NOVAOS ARCHITECT ]".to_owned(),
                "Lead Systems Engineer • NovaOS Systems Lab".to_owned(),
                "&quot;In NovaOS 2.0, every single line of kernel, window manager and browser is pure Rust.&quot;".to_owned(),
                "#ff5500",
                "NOVA",
            )
        } else if v.title.contains("Parliament") || (v.channel.contains("ATN") && v.title.contains("Mugdho")) {
            (
                "● LIVE 1080p60 HD • ATN News National Broadcast",
                "📺 NATIONAL PARLIAMENT REPORT",
                "Mugdho Puro Sangsad: Special Parliamentary Session".to_owned(),
                [
                    "• Historic national assembly debate &amp; reform bills",
                    "• Prime time political analysis &amp; floor speeches",
                    "• Broadcast live from Jatiya Sangsad Bhaban",
                ],
                "[ 📡 ATN NEWS PRIME ]".to_owned(),
                "Senior Parliamentary Correspondent • ATN".to_owned(),
                "&quot;Jatiya Sangsade shokol duler shodoshyoder uposthitite guruttopurno bil pass holo.&quot;".to_owned(),
                "#dc2626",
                "ATN",
            )
        } else if v.title.contains("Mashrafe") {
            (
                "● LIVE 1080p60 HD • Sports Exclusive Interview",
                "🏏 BANGLADESH CRICKET SPECIAL",
                "Dirghodin Por Dekha Mileche • Mashrafe Mortaza".to_owned(),
                [
                    "• In-depth interview on Bangladesh cricket journey",
                    "• Career reflections, leadership &amp; youth inspiration",
                    "• Khabar Protidin Special Sports Presentation",
                ],
                "[ 🏆 MASHRAFE MORTAZA ]".to_owned(),
                "Former Captain • Bangladesh National Cricket Team".to_owned(),
                "&quot;Desher hoye math-e namar je onubhuti, tar shathe kono kichur tulona hoy na.&quot;".to_owned(),
                "#16a34a",
                "KP",
            )
        } else if v.title.contains("Barita") {
            (
                "● LIVE 1080p60 HD • Investigative Report • ATN News",
                "📰 SPECIAL INVESTIGATION",
                "Barita Aji Chere Dite Hobe • Exclusive Report".to_owned(),
                [
                    "• Special investigative journalism on urban housing",
                    "• Ground realities &amp; public safety inspection",
                    "• Comprehensive reporting by ATN News Team",
                ],
                "[ 🎙 INVESTIGATIVE DESK ]".to_owned(),
                "Special Investigation Unit • ATN News".to_owned(),
                "&quot;Ghotona sthol theke amader bishes protibedok janacchen bishes tothyo.&quot;".to_owned(),
                "#dc2626",
                "ATN",
            )
        } else if v.title.contains("Biyer Purbe") || v.video_id == "rel_marriage" {
            (
                "● LIVE 1080p60 HD • Bayyinah Islamic Wisdom • NAK HD",
                "📖 ISLAMIC COUNSELING &amp; WISDOM",
                "Biyer Purbe Kichu Proyojonio Poramosho • Nouman Ali Khan".to_owned(),
                [
                    "• Foundations of a blissful and resilient marriage",
                    "• Emotional maturity, realistic expectations &amp; empathy",
                    "• Quranic guidance for youth embarking on matrimony",
                ],
                "[ 🎙 DR. NOUMAN ALI KHAN ]".to_owned(),
                "Speaker &amp; Founder • Bayyinah Institute".to_owned(),
                "&quot;Marriage requires immense patience, emotional maturity, and mutual respect every single day.&quot;".to_owned(),
                "#059669",
                "NAK",
            )
        } else {
            (
                "● LIVE 1080p60 HD • Spektrum Media Official Stream • NAK HD",
                "📖 BAYYINAH LECTURE SERIES",
                v.title.clone(),
                [
                    "• Building spiritual resilience in modern times",
                    "• Overcoming toxic digital distractions",
                    "• Achieving balance: Faith, Education &amp; Moral Character",
                ],
                "[ 🎙 DR. NOUMAN ALI KHAN ]".to_owned(),
                "Speaker &amp; Founder • Bayyinah Institute".to_owned(),
                "&quot;Our purpose in life is building character, spiritual resilience and moral clarity.&quot;".to_owned(),
                "#cc0000",
                "NAK",
            )
        }
    }

    /// Builds the clean initial landing page matching Image 1 & 2 in White Mode (Light Theme).
    #[cfg(feature = "std")]
    pub fn build_youtube_clean_home_html(
        query_input: &str,
        suggestions_open: bool,
        is_focused: bool,
        cursor_visible: bool,
    ) -> String {
        let (search_border, display_input) = if is_focused {
            let cursor = if cursor_visible { "|" } else { " " };
            let text = if query_input.is_empty() {
                format!("<span style=\"color:#065fd4; font-weight:bold;\">{}</span><span style=\"color:#888888;\">Search</span>", cursor)
            } else {
                format!("{}<span style=\"color:#065fd4; font-weight:bold;\">{}</span>", query_input, cursor)
            };
            ("1.5px solid #065fd4", text)
        } else {
            let text = if query_input.is_empty() {
                "<span style=\"color:#888888;\">Search</span>".to_owned()
            } else {
                query_input.to_owned()
            };
            ("1px solid #cccccc", text)
        };

        let sugg_display = if suggestions_open || is_focused { "block" } else { "none" };
        let base_q = if query_input.is_empty() { "naki in bangla" } else { query_input };

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>YouTube</title>
<style>
body {{
    background-color: #ffffff;
    color: #0f0f0f;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
    padding: 10px 18px;
    display: block;
}}
.hamburger {{
    display: inline-block;
    font-size: 18px;
    color: #0f0f0f;
    width: 28px;
    cursor: pointer;
    vertical-align: middle;
}}
.logo {{
    display: inline-block;
    font-size: 18px;
    font-weight: bold;
    color: #0f0f0f;
    width: 130px;
    cursor: pointer;
    vertical-align: middle;
}}
.logo-badge {{
    font-size: 10px;
    color: #606060;
    margin-left: 2px;
}}
.search-container {{
    display: inline-block;
    width: 540px;
    margin-left: 20px;
    vertical-align: middle;
}}
.search-pill {{
    background-color: #ffffff;
    border-right: none;
    border-radius: 20px 0 0 20px;
    padding: 8px 16px;
    width: 380px;
    color: #0f0f0f;
    font-size: 13px;
    display: inline-block;
    vertical-align: middle;
}}
.search-btn {{
    background-color: #f8f8f8;
    border: 1px solid #cccccc;
    border-radius: 0 20px 20px 0;
    width: 56px;
    padding: 8px 0;
    text-align: center;
    color: #0f0f0f;
    font-size: 14px;
    display: inline-block;
    vertical-align: middle;
    cursor: pointer;
}}
.mic-btn {{
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    border-radius: 50%;
    width: 36px;
    height: 36px;
    display: inline-block;
    margin-left: 10px;
    text-align: center;
    color: #0f0f0f;
    font-size: 14px;
    vertical-align: middle;
    cursor: pointer;
}}
.signin-btn {{
    display: inline-block;
    float: right;
    border: 1px solid #065fd4;
    color: #065fd4;
    background-color: #ffffff;
    border-radius: 18px;
    padding: 7px 16px;
    font-size: 13px;
    font-weight: bold;
    margin-top: 2px;
    vertical-align: middle;
}}
.sugg-dropdown {{
    position: absolute;
    top: 50px;
    left: 218px;
    width: 440px;
    background-color: #ffffff;
    border: 1px solid #e5e5e5;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.12);
    display: {};
    padding: 8px 0;
    z-index: 100;
}}
.sugg-item {{
    padding: 8px 18px;
    font-size: 13px;
    color: #0f0f0f;
    cursor: pointer;
}}
.sugg-footer {{
    border-top: 1px solid #f2f2f2;
    padding: 6px 18px;
    font-size: 11px;
    color: #909090;
    text-align: right;
}}
.layout-row {{
    display: block;
}}
.sidebar {{
    display: inline-block;
    width: 170px;
    background-color: #ffffff;
    border-right: 1px solid #e5e5e5;
    padding-top: 12px;
    vertical-align: top;
}}
.side-item {{
    padding: 10px 18px;
    font-size: 13px;
    color: #0f0f0f;
    border-radius: 10px;
    margin: 2px 10px;
    cursor: pointer;
}}
.side-active {{
    background-color: #f2f2f2;
    font-weight: bold;
}}
.side-divider {{
    border-top: 1px solid #e5e5e5;
    margin: 10px 14px;
}}
.signin-prompt {{
    padding: 12px 18px;
    font-size: 12px;
    color: #606060;
    line-height: 1.4;
}}
.content-area {{
    display: inline-block;
    width: 790px;
    vertical-align: top;
    padding: 36px 20px;
}}
.welcome-card {{
    background-color: #f9f9f9;
    border: 1px solid #e5e5e5;
    border-radius: 16px;
    padding: 38px 24px;
    text-align: center;
    max-width: 520px;
    margin: 16px auto 26px auto;
}}
.welcome-title {{
    font-size: 21px;
    font-weight: bold;
    color: #0f0f0f;
    margin-bottom: 10px;
}}
.welcome-sub {{
    font-size: 13px;
    color: #606060;
    line-height: 1.5;
}}
.quick-pills {{
    text-align: center;
    margin-top: 16px;
}}
.pill {{
    display: inline-block;
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    color: #0f0f0f;
    padding: 7px 16px;
    border-radius: 16px;
    font-size: 12px;
    margin: 6px;
    cursor: pointer;
}}
.pill-active {{
    background-color: #0f0f0f;
    color: #ffffff;
    font-weight: bold;
}}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\"><span style=\"color:#ff0000;\">▶</span> YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\" style=\"border:{}; border-right:none;\">{}</div>
        <div class=\"search-btn\">🔍</div>
        <div class=\"mic-btn\">🎙</div>
    </div>
    <div class=\"signin-btn\">👤 Sign in</div>
</div>

<div class=\"sugg-dropdown\">
    <div class=\"sugg-item\">🔍 <b>{}</b></div>
    <div class=\"sugg-item\">🔍 {} natok</div>
    <div class=\"sugg-item\">🔍 {} song official</div>
    <div class=\"sugg-item\">🔍 {} waz 2026</div>
    <div class=\"sugg-footer\">Report search predictions</div>
</div>

<div class=\"layout-row\">
    <div class=\"sidebar\">
        <div class=\"side-item side-active\">[🏠 Home]</div>
        <div class=\"side-item\">[⚡ Shorts]</div>
        <div class=\"side-item\">[📺 Subscriptions]</div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[👤 You]</div>
        <div class=\"side-item\">[🕒 History]</div>
        <div class=\"side-divider\"></div>
        <div class=\"signin-prompt\">Sign in to like videos,<br>comment, and subscribe.<br><span style=\"color:#065fd4; font-weight:bold;\">Sign in</span></div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[🎵 Music]</div>
        <div class=\"side-item\">[🎮 Gaming]</div>
        <div class=\"side-item\">[📰 News]</div>
        <div class=\"side-item\">[💡 Podcasts]</div>
    </div>

    <div class=\"content-area\">
        <div class=\"welcome-card\">
            <div class=\"welcome-title\">Try searching to get started</div>
            <div class=\"welcome-sub\">Start watching videos to help us build a feed of videos you'll love.</div>
        </div>

        <div class=\"quick-pills\">
            <div class=\"pill pill-active\">🔍 {}</div>
            <div class=\"pill\">Explore Trending Feed</div>
            <div class=\"pill\">Popular Music</div>
            <div class=\"pill\">Gaming Live</div>
            <div class=\"pill\">Tech &amp; Coding</div>
        </div>
    </div>
</div>
</body>
</html>",
            sugg_display, search_border, display_input, base_q, base_q, base_q, base_q, base_q
        )
    }

    /// Builds the authentic search results page matching Image 3 in White Mode.
    #[cfg(feature = "std")]
    pub fn build_youtube_search_results_html(
        query: &str,
        videos: &[YouTubeVideo],
        query_input: &str,
        is_focused: bool,
        cursor_visible: bool,
    ) -> String {
        let (search_border, display_input) = if is_focused {
            let cursor = if cursor_visible { "|" } else { " " };
            let text = if query_input.is_empty() {
                format!("<span style=\"color:#065fd4; font-weight:bold;\">{}</span><span style=\"color:#888888;\">Search</span>", cursor)
            } else {
                format!("{}<span style=\"color:#065fd4; font-weight:bold;\">{}</span>", query_input, cursor)
            };
            ("1.5px solid #065fd4", text)
        } else {
            let text = if query_input.is_empty() {
                query.to_owned()
            } else {
                query_input.to_owned()
            };
            ("1px solid #cccccc", text)
        };

        let mut cards = String::new();
        for v in videos {
            let (bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, avatar_bg, avatar_text) =
                Self::get_video_thumbnail_details(v);

            cards.push_str(&format!(
                "<div class=\"result-card\">
                    <div class=\"thumb-widescreen\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"thumb-tag\" style=\"background-color:{}; color:{};\">{}</div>
                        <div class=\"thumb-headline\">{}</div>
                        <div class=\"thumb-subtext\">{}</div>
                        <div class=\"dur-badge\">{}</div>
                    </div>
                    <div class=\"result-info\">
                        <div class=\"res-title\">{}</div>
                        <div class=\"res-meta\">{} • {}</div>
                        <div class=\"res-channel\"><span class=\"ch-avatar\" style=\"background-color:{}; color:#ffffff;\">{}</span> {} <span class=\"verified\">✔</span></div>
                        <div class=\"res-desc\">{}</div>
                    </div>
                </div>",
                bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, v.duration,
                v.title, v.views, v.published, avatar_bg, avatar_text, v.channel, v.description
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - YouTube</title>
<style>
body {{
    background-color: #ffffff;
    color: #0f0f0f;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
    padding: 10px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 18px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 18px; font-weight: bold; color: #0f0f0f; width: 130px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 540px; margin-left: 20px; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 8px 16px; width: 380px; color: #0f0f0f; font-size: 13px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 56px; padding: 8px 0; text-align: center; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 36px; height: 36px; display: inline-block; margin-left: 10px; text-align: center; color: #0f0f0f; font-size: 14px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 7px 16px; font-size: 13px; font-weight: bold; margin-top: 2px; vertical-align: middle; }}
.layout-row {{ display: block; }}
.sidebar {{ display: inline-block; width: 170px; background-color: #ffffff; border-right: 1px solid #e5e5e5; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #0f0f0f; border-radius: 10px; margin: 2px 10px; cursor: pointer; }}
.side-active {{ background-color: #f2f2f2; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #e5e5e5; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 790px; vertical-align: top; padding: 14px 18px; }}
.result-card {{ display: block; margin-bottom: 22px; cursor: pointer; }}
.thumb-widescreen {{ display: inline-block; width: 280px; height: 135px; border-radius: 10px; vertical-align: top; position: relative; padding: 6px; box-sizing: border-box; }}
.thumb-tag {{ font-size: 10px; font-weight: bold; padding: 3px 6px; border-radius: 6px; display: inline-block; }}
.thumb-headline {{ color: #ffffff; font-size: 13px; font-weight: bold; padding-top: 16px; text-align: center; line-height: 1.3; }}
.thumb-subtext {{ color: #e2e8f0; font-size: 10px; text-align: center; margin-top: 6px; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 2px 5px; font-size: 10px; font-weight: bold; float: right; margin: 4px; }}
.result-info {{ display: inline-block; width: 470px; padding-left: 16px; vertical-align: top; }}
.res-title {{ font-size: 15px; font-weight: bold; color: #0f0f0f; line-height: 1.3; margin-bottom: 6px; }}
.res-meta {{ font-size: 12px; color: #606060; margin-bottom: 6px; }}
.res-channel {{ font-size: 12px; color: #606060; margin-bottom: 8px; }}
.ch-avatar {{ border-radius: 50%; padding: 2px 6px; margin-right: 4px; font-weight: bold; font-size: 10px; }}
.verified {{ color: #606060; font-size: 11px; }}
.res-desc {{ font-size: 11px; color: #606060; line-height: 1.4; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\"><span style=\"color:#ff0000;\">▶</span> YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\" style=\"border:{}; border-right:none;\">{}</div>
        <div class=\"search-btn\">🔍</div>
        <div class=\"mic-btn\">🎙</div>
    </div>
    <div class=\"signin-btn\">👤 Sign in</div>
</div>

<div class=\"layout-row\">
    <div class=\"sidebar\">
        <div class=\"side-item side-active\">[🏠 Home]</div>
        <div class=\"side-item\">[⚡ Shorts]</div>
        <div class=\"side-item\">[📺 Subscriptions]</div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[👤 You]</div>
        <div class=\"side-item\">[🕒 History]</div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[🎵 Music]</div>
        <div class=\"side-item\">[🎮 Gaming]</div>
        <div class=\"side-item\">[📰 News]</div>
    </div>

    <div class=\"content-area\">
        {}
    </div>
</div>
</body>
</html>",
            query, search_border, display_input, cards
        )
    }

    /// Builds the authentic Cinema Player watch page matching Image 4 in White Mode (Light Theme).
    #[cfg(feature = "std")]
    pub fn build_youtube_watch_html(
        video: &YouTubeVideo,
        related: &[YouTubeVideo],
        is_playing: bool,
        is_subscribed: bool,
        likes: u32,
        elapsed_sec: u32,
        anim_frame: usize,
        query_input: &str,
        is_focused: bool,
        cursor_visible: bool,
    ) -> String {
        let (hud_badge, slide_tag, slide_main, bullets, speaker_title, speaker_sub, cc_subtitle, avatar_bg, avatar_text) =
            Self::get_player_stage_details(video);

        let total_sec = Self::parse_duration_to_sec(&video.duration);
        let cur_sec = elapsed_sec.min(total_sec);
        let time_str = format!("{:02}:{:02}", cur_sec / 60, cur_sec % 60);
        let pct = ((cur_sec as f32 / total_sec as f32) * 100.0).clamp(2.0, 98.0);
        let timeline_width = format!("{:.1}%", pct);

        let spectrum_frames = [
            "SPECTRUM: ▂▃▅▆▇█▇▆▅▃▂ 100%",
            "SPECTRUM: ▃▅▆▇█▇▆▅▃▂▃ 100%",
            "SPECTRUM: ▅▆▇█▇▆▅▃▂▃▅ 100%",
            "SPECTRUM: ▆▇█▇▆▅▃▂▃▅▆ 100%",
            "SPECTRUM: █▇▆▅▃▂▃▅▆▇█ 100%",
            "SPECTRUM: ▇▆▅▃▂▃▅▆▇█▇ 100%",
        ];
        let spectrum_str = if is_playing {
            spectrum_frames[anim_frame % spectrum_frames.len()]
        } else {
            "SPECTRUM: ▂▂▂▂▂▂▂▂▂▂▂ PAUSED"
        };

        let play_badge = if is_playing {
            let dot = if (anim_frame % 2) == 0 { "●" } else { "○" };
            format!("<span style=\"color:#22c55e; font-weight:bold;\">{} LIVE STREAM ACTIVE</span>", dot)
        } else {
            "<span style=\"color:#f59e0b; font-weight:bold;\">⏸ PAUSED</span>".to_owned()
        };
        let play_btn = if is_playing { "⏸ Pause" } else { "▶ Play" };
        let sub_btn = if is_subscribed { "Subscribed ✓" } else { "Subscribe" };
        let sub_bg = if is_subscribed { "#f2f2f2" } else { "#cc0000" };
        let sub_text_color = if is_subscribed { "#0f0f0f" } else { "#ffffff" };

        let (search_border, display_input) = if is_focused {
            let cursor = if cursor_visible { "|" } else { " " };
            let text = if query_input.is_empty() {
                format!("<span style=\"color:#065fd4; font-weight:bold;\">{}</span><span style=\"color:#888888;\">Search</span>", cursor)
            } else {
                format!("{}<span style=\"color:#065fd4; font-weight:bold;\">{}</span>", query_input, cursor)
            };
            ("1.5px solid #065fd4", text)
        } else {
            let text = if query_input.is_empty() {
                "Search".to_owned()
            } else {
                query_input.to_owned()
            };
            ("1px solid #cccccc", text)
        };

        let mut rel_cards = String::new();
        for r in related {
            let (rbg, rborder, _, _, rtag, _, _, _, _) = Self::get_video_thumbnail_details(r);
            rel_cards.push_str(&format!(
                "<div class=\"rel-item\">
                    <div class=\"rel-thumb\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"dur-badge\">{}</div>
                        <div style=\"color:#ffffff; font-size:9px; font-weight:bold; padding:4px 6px;\">{}</div>
                        <div style=\"color:#e2e8f0; font-size:11px; font-weight:bold; text-align:center; padding-top:6px;\">▶ HD</div>
                    </div>
                    <div class=\"rel-info\">
                        <div class=\"rel-title\">{}</div>
                        <div class=\"rel-channel\">{} ✔</div>
                        <div class=\"rel-meta\">{} • {}</div>
                    </div>
                </div>",
                rbg, rborder, r.duration, rtag, r.title, r.channel, r.views, r.published
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - YouTube</title>
<style>
body {{
    background-color: #ffffff;
    color: #0f0f0f;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
    padding: 10px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 18px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 18px; font-weight: bold; color: #0f0f0f; width: 130px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 540px; margin-left: 20px; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 8px 16px; width: 380px; color: #0f0f0f; font-size: 13px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 56px; padding: 8px 0; text-align: center; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 36px; height: 36px; display: inline-block; margin-left: 10px; text-align: center; color: #0f0f0f; font-size: 14px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 7px 16px; font-size: 13px; font-weight: bold; margin-top: 2px; vertical-align: middle; }}
.watch-container {{ padding: 14px 18px; display: block; }}
.player-column {{ display: inline-block; width: 610px; vertical-align: top; }}
.cinema-player {{
    background-color: #0b0f19;
    border-radius: 12px;
    border: 1px solid #1e293b;
    display: block;
}}
.player-hud-top {{
    background-color: #0f172a;
    color: #94a3b8;
    font-size: 11px;
    padding: 6px 14px;
    border-radius: 12px 12px 0 0;
}}
.player-stage {{
    padding: 8px 10px;
    display: block;
}}
.stage-slide {{
    display: inline-block;
    width: 320px;
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 8px 10px;
    vertical-align: top;
}}
.slide-tag {{
    color: #38bdf8;
    font-size: 10px;
    font-weight: bold;
    margin-bottom: 4px;
}}
.slide-main {{
    color: #ffffff;
    font-size: 14px;
    font-weight: bold;
    margin-bottom: 6px;
}}
.slide-bullets {{
    color: #cbd5e1;
    font-size: 11px;
    line-height: 1.5;
}}
.stage-speaker {{
    display: inline-block;
    width: 220px;
    background-color: #18181b;
    border: 1px solid #27272a;
    border-radius: 8px;
    padding: 8px 10px;
    vertical-align: top;
    margin-left: 6px;
}}
.speaker-title {{
    color: #ffffff;
    font-size: 12px;
    font-weight: bold;
    margin-bottom: 4px;
}}
.speaker-sub {{
    color: #a1a1aa;
    font-size: 10px;
    margin-bottom: 6px;
}}
.audio-spectrum {{
    color: #38bdf8;
    font-size: 11px;
    margin-bottom: 4px;
}}
.stage-state {{
    font-size: 11px;
    margin-top: 4px;
}}
.stage-cc {{
    text-align: center;
    margin-top: 8px;
    margin-bottom: 6px;
}}
.cc-pill {{
    background-color: #1e293b;
    color: #f8fafc;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 4px 12px;
    font-size: 11px;
    display: inline-block;
}}
.timeline-bar {{
    background-color: #333333;
    height: 5px;
    width: 100%;
    position: relative;
    display: block;
}}
.timeline-fill {{
    background-color: #ff0000;
    height: 5px;
    display: inline-block;
}}
.timeline-dot {{
    color: #ff0000;
    font-size: 12px;
    display: inline-block;
    margin-left: -4px;
    vertical-align: middle;
}}
.player-controls {{
    background-color: #0f0f0f;
    padding: 8px 14px;
    font-size: 12px;
    color: #f1f1f1;
    border-radius: 0 0 12px 12px;
}}
.ctrl-item {{ display: inline-block; margin-right: 12px; cursor: pointer; }}
.float-r {{ float: right; margin-left: 10px; margin-right: 0; }}
.shield-badge {{ display: inline-block; color: #ff5500; font-weight: bold; font-size: 11px; margin-left: 8px; }}
.vid-heading {{ font-size: 17px; font-weight: bold; color: #0f0f0f; margin-top: 14px; line-height: 1.3; }}
.channel-row {{ margin-top: 12px; padding-bottom: 12px; border-bottom: 1px solid #e5e5e5; display: block; }}
.ch-avatar-big {{
    border-radius: 50%;
    width: 36px;
    height: 36px;
    display: inline-block;
    text-align: center;
    line-height: 36px;
    font-weight: bold;
    font-size: 11px;
    vertical-align: middle;
}}
.ch-details {{ display: inline-block; vertical-align: middle; margin-left: 10px; width: 170px; }}
.ch-name {{ font-size: 13px; font-weight: bold; color: #0f0f0f; }}
.ch-subs {{ font-size: 11px; color: #606060; }}
.sub-pill {{
    display: inline-block;
    background-color: {};
    color: {};
    font-weight: bold;
    padding: 7px 16px;
    border-radius: 18px;
    font-size: 12px;
    vertical-align: middle;
    margin-left: 8px;
    border: 1px solid #cccccc;
    cursor: pointer;
}}
.action-pills {{ display: inline-block; float: right; vertical-align: middle; margin-top: 4px; }}
.action-pill {{
    display: inline-block;
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    border-radius: 18px;
    padding: 6px 12px;
    font-size: 11px;
    color: #0f0f0f;
    margin-left: 4px;
    cursor: pointer;
}}
.desc-box {{
    background-color: #f2f2f2;
    border-radius: 12px;
    padding: 12px 16px;
    margin-top: 12px;
    font-size: 12px;
    color: #0f0f0f;
    line-height: 1.5;
}}
.related-column {{
    display: inline-block;
    width: 330px;
    vertical-align: top;
    margin-left: 18px;
}}
.rel-header {{ font-size: 14px; font-weight: bold; color: #0f0f0f; margin-bottom: 12px; }}
.rel-item {{ display: block; margin-bottom: 14px; cursor: pointer; }}
.rel-thumb {{ display: inline-block; width: 120px; height: 68px; border-radius: 8px; vertical-align: top; position: relative; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 2px 4px; font-size: 10px; font-weight: bold; float: right; margin: 4px; }}
.rel-info {{ display: inline-block; width: 185px; padding-left: 10px; vertical-align: top; }}
.rel-title {{ font-size: 12px; font-weight: bold; color: #0f0f0f; line-height: 1.3; margin-bottom: 4px; }}
.rel-channel {{ font-size: 11px; color: #606060; margin-bottom: 2px; }}
.rel-meta {{ font-size: 10px; color: #606060; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\"><span style=\"color:#ff0000;\">▶</span> YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\" style=\"border:{}; border-right:none;\">{}</div>
        <div class=\"search-btn\">🔍</div>
        <div class=\"mic-btn\">🎙</div>
    </div>
    <div class=\"signin-btn\">👤 Sign in</div>
</div>

<div class=\"watch-container\">
    <div class=\"player-column\">
        <div class=\"cinema-player\">
            <div class=\"player-hud-top\">
                <span style=\"color:#22c55e; font-weight:bold;\">{}</span> • Aero Cinema Stream Engine
            </div>
            <div class=\"player-stage\">
                <div class=\"stage-slide\">
                    <div class=\"slide-tag\">{}</div>
                    <div class=\"slide-main\">{}</div>
                    <div class=\"slide-bullets\">
                        {}<br>
                        {}<br>
                        {}
                    </div>
                </div>
                <div class=\"stage-speaker\">
                    <div class=\"speaker-title\">{}</div>
                    <div class=\"speaker-sub\">{}</div>
                    <div class=\"audio-spectrum\">{}</div>
                    <div class=\"stage-state\">{}</div>
                </div>
                <div class=\"stage-cc\">
                    <span class=\"cc-pill\">[ CC: {} ]</span>
                </div>
            </div>
            <div class=\"timeline-bar\">
                <div class=\"timeline-fill\" style=\"width:{};\"></div>
                <div class=\"timeline-dot\">●</div>
            </div>
            <div class=\"player-controls\">
                <div class=\"ctrl-item\">{}</div>
                <div class=\"ctrl-item\">⏮</div>
                <div class=\"ctrl-item\">⏭</div>
                <div class=\"ctrl-item\">🔊 100%</div>
                <div class=\"ctrl-item\">{} / {}</div>
                <div class=\"shield-badge\">🛡️ Brave Shields: 0 Video Ads</div>
                <div class=\"ctrl-item float-r\">⛶ Fullscreen</div>
                <div class=\"ctrl-item float-r\">⚙ 1080p HD</div>
                <div class=\"ctrl-item float-r\">CC</div>
            </div>
        </div>

        <div class=\"vid-heading\">{}</div>

        <div class=\"channel-row\">
            <div class=\"ch-avatar-big\" style=\"background-color:{}; color:#ffffff;\">{}</div>
            <div class=\"ch-details\">
                <div class=\"ch-name\">{} ✔</div>
                <div class=\"ch-subs\">779K subscribers</div>
            </div>
            <div class=\"sub-pill\">{}</div>
            <div class=\"action-pills\">
                <div class=\"action-pill\">👍 {}</div>
                <div class=\"action-pill\">↗ Share</div>
                <div class=\"action-pill\">📥 Download</div>
                <div class=\"action-pill\">• • •</div>
            </div>
        </div>

        <div class=\"desc-box\">
            <b>{} views • {}</b><br>
            {}
        </div>
    </div>

    <div class=\"related-column\">
        <div class=\"rel-header\">Up Next</div>
        {}
    </div>
</div>
</body>
</html>",
            video.title,
            sub_bg,
            sub_text_color,
            search_border,
            display_input,
            hud_badge,
            slide_tag,
            slide_main,
            bullets[0],
            bullets[1],
            bullets[2],
            speaker_title,
            speaker_sub,
            spectrum_str,
            play_badge,
            cc_subtitle,
            timeline_width,
            play_btn,
            time_str,
            video.duration,
            video.title,
            avatar_bg,
            avatar_text,
            video.channel,
            sub_btn,
            likes,
            video.views,
            video.published,
            video.description,
            rel_cards
        )
    }

    /// Builds the rich Home Feed matching Image 5 in White Mode.
    #[cfg(feature = "std")]
    pub fn build_youtube_feed_html(
        category: &str,
        videos: &[YouTubeVideo],
        query_input: &str,
        is_focused: bool,
        cursor_visible: bool,
    ) -> String {
        let (search_border, display_input) = if is_focused {
            let cursor = if cursor_visible { "|" } else { " " };
            let text = if query_input.is_empty() {
                format!("<span style=\"color:#065fd4; font-weight:bold;\">{}</span><span style=\"color:#888888;\">Search</span>", cursor)
            } else {
                format!("{}<span style=\"color:#065fd4; font-weight:bold;\">{}</span>", query_input, cursor)
            };
            ("1.5px solid #065fd4", text)
        } else {
            let text = if query_input.is_empty() {
                "Search".to_owned()
            } else {
                query_input.to_owned()
            };
            ("1px solid #cccccc", text)
        };

        let mut cards = String::new();
        for v in videos {
            let title_display = if v.title.chars().count() > 28 {
                let s: String = v.title.chars().take(26).collect();
                format!("{}...", s)
            } else {
                v.title.clone()
            };
            let (bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, avatar_bg, avatar_text) =
                Self::get_video_thumbnail_details(v);

            cards.push_str(&format!(
                "<div class=\"grid-card\">
                    <div class=\"card-thumb\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"thumb-tag\" style=\"background-color:{}; color:{};\">{}</div>
                        <div class=\"thumb-headline\">{}</div>
                        <div class=\"thumb-subtext\">{}</div>
                        <div class=\"dur-badge\">{}</div>
                    </div>
                    <div class=\"card-body\">
                        <div class=\"card-avatar\" style=\"background-color:{}; color:#ffffff;\">{}</div>
                        <div class=\"card-text\">
                            <div class=\"card-title\">{}</div>
                            <div class=\"card-channel\">{} ✔</div>
                            <div class=\"card-meta\">{} • {}</div>
                        </div>
                    </div>
                </div>",
                bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, v.duration,
                avatar_bg, avatar_text, title_display, v.channel, v.views, v.published
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>YouTube</title>
<style>
body {{
    background-color: #ffffff;
    color: #0f0f0f;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #ffffff;
    border-bottom: 1px solid #e5e5e5;
    padding: 10px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 18px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 18px; font-weight: bold; color: #0f0f0f; width: 130px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 540px; margin-left: 20px; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 8px 16px; width: 380px; color: #0f0f0f; font-size: 13px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 56px; padding: 8px 0; text-align: center; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 36px; height: 36px; display: inline-block; margin-left: 10px; text-align: center; color: #0f0f0f; font-size: 14px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 7px 16px; font-size: 13px; font-weight: bold; margin-top: 2px; vertical-align: middle; }}
.layout-row {{ display: block; }}
.sidebar {{ display: inline-block; width: 170px; background-color: #ffffff; border-right: 1px solid #e5e5e5; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #0f0f0f; border-radius: 10px; margin: 2px 10px; cursor: pointer; }}
.side-active {{ background-color: #f2f2f2; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #e5e5e5; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 790px; vertical-align: top; padding: 14px 18px; }}
.chips-bar {{ margin-bottom: 18px; display: block; }}
.chip {{
    display: inline-block;
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    color: #0f0f0f;
    padding: 6px 14px;
    border-radius: 16px;
    font-size: 12px;
    margin-right: 6px;
    margin-bottom: 6px;
    cursor: pointer;
}}
.chip-active {{ background-color: #0f0f0f; color: #ffffff; font-weight: bold; }}
.grid-card {{
    display: inline-block;
    width: 240px;
    margin: 6px 8px 20px 8px;
    vertical-align: top;
    cursor: pointer;
}}
.card-thumb {{
    border-radius: 10px;
    height: 135px;
    position: relative;
    padding: 6px;
    box-sizing: border-box;
    display: block;
}}
.thumb-tag {{ font-size: 10px; font-weight: bold; padding: 3px 6px; border-radius: 6px; display: inline-block; }}
.thumb-headline {{ color: #ffffff; font-size: 13px; font-weight: bold; padding-top: 16px; text-align: center; line-height: 1.3; }}
.thumb-subtext {{ color: #e2e8f0; font-size: 10px; text-align: center; margin-top: 6px; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 2px 5px; font-size: 10px; font-weight: bold; float: right; margin: 4px; }}
.card-body {{ margin-top: 10px; display: block; }}
.card-avatar {{
    border-radius: 50%;
    width: 32px;
    height: 32px;
    display: inline-block;
    text-align: center;
    line-height: 32px;
    font-size: 10px;
    font-weight: bold;
    vertical-align: top;
}}
.card-text {{ display: inline-block; width: 200px; padding-left: 8px; vertical-align: top; }}
.card-title {{ font-size: 13px; font-weight: bold; color: #0f0f0f; line-height: 1.3; margin-bottom: 4px; }}
.card-channel {{ font-size: 11px; color: #606060; margin-bottom: 2px; }}
.card-meta {{ font-size: 10px; color: #606060; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\"><span style=\"color:#ff0000;\">▶</span> YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\" style=\"border:{}; border-right:none;\">{}</div>
        <div class=\"search-btn\">🔍</div>
        <div class=\"mic-btn\">🎙</div>
    </div>
    <div class=\"signin-btn\">👤 Sign in</div>
</div>

<div class=\"layout-row\">
    <div class=\"sidebar\">
        <div class=\"side-item side-active\">[🏠 Home]</div>
        <div class=\"side-item\">[⚡ Shorts]</div>
        <div class=\"side-item\">[📺 Subscriptions]</div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[👤 You]</div>
        <div class=\"side-item\">[🕒 History]</div>
        <div class=\"side-divider\"></div>
        <div class=\"side-item\">[🎵 Music]</div>
        <div class=\"side-item\">[🎮 Gaming]</div>
        <div class=\"side-item\">[📰 News]</div>
    </div>

    <div class=\"content-area\">
        <div class=\"chips-bar\">
            <div class=\"chip chip-active\">All</div>
            <div class=\"chip\">News</div>
            <div class=\"chip\">Podcasts</div>
            <div class=\"chip\">Music</div>
            <div class=\"chip\">Gaming</div>
            <div class=\"chip\">Live</div>
            <div class=\"chip\">Bangla</div>
            <div class=\"chip\">Coding</div>
        </div>

        <div class=\"grid-feed\">
            {}
        </div>
    </div>
</div>
</body>
</html>",
            search_border, display_input, cards
        )
    }

    #[cfg(feature = "std")]
    pub fn build_youtube_page_html(
        url_str: &str,
        query_input: &str,
        is_focused: bool,
        cursor_visible: bool,
        is_playing: bool,
        is_subscribed: bool,
        likes: u32,
        elapsed_sec: u32,
        anim_frame: usize,
    ) -> (String, String) {
        let trimmed = url_str.trim();

        // 1. Watch page: /watch?v=...
        if trimmed.contains("/watch") || trimmed.contains("watch?v=") {
            let vid = if let Some(v_part) = trimmed.split("v=").nth(1) {
                v_part.split('&').next().unwrap_or("4NlKj7Saogc")
            } else {
                "4NlKj7Saogc"
            };
            let video = Self::get_video_by_id(vid);
            let related = Self::get_related_videos_for(vid);
            let html = Self::build_youtube_watch_html(
                &video,
                &related,
                is_playing,
                is_subscribed,
                likes,
                elapsed_sec,
                anim_frame,
                query_input,
                is_focused,
                cursor_visible,
            );
            return (format!("{} - YouTube", video.title), html);
        }

        // 2. Search results page: search_query=... or ?q=...
        if trimmed.contains("search_query=") || trimmed.contains("?q=") {
            let q = if let Some(q_part) = trimmed.split("search_query=").nth(1) {
                q_part.split('&').next().unwrap_or("naki in bangla")
            } else if let Some(q_part) = trimmed.split("?q=").nth(1) {
                q_part.split('&').next().unwrap_or("naki in bangla")
            } else {
                "naki in bangla"
            };
            let decoded_q = q.replace('+', " ");
            let videos = Self::get_default_search_results(&decoded_q);
            let html = Self::build_youtube_search_results_html(
                &decoded_q,
                &videos,
                if query_input.is_empty() { &decoded_q } else { query_input },
                is_focused,
                cursor_visible,
            );
            return (format!("{} - YouTube", decoded_q), html);
        }

        // 3. Home Feed with Filter Chips: feed=true
        if trimmed.contains("feed=true") || trimmed.contains("/feed") {
            let videos = Self::get_default_home_feed();
            let html = Self::build_youtube_feed_html("All", &videos, query_input, is_focused, cursor_visible);
            return ("YouTube".to_owned(), html);
        }

        // 4. Clean Landing Page (Image 1 & 2): default or clean=true
        let raw_q = if !query_input.is_empty() {
            query_input
        } else if trimmed.contains("q=") {
            trimmed.split("q=").nth(1).unwrap_or("").split('&').next().unwrap_or("")
        } else {
            ""
        };
        let q_input = raw_q.replace('+', " ");
        let sugg_open = trimmed.contains("sugg=true") || is_focused;
        let html = Self::build_youtube_clean_home_html(&q_input, sugg_open, is_focused, cursor_visible);
        ("YouTube".to_owned(), html)
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
                return Self::build_youtube_page_html(trimmed, "", false, false, true, false, 4300, 134, 0);
            }
            #[cfg(not(feature = "std"))]
            {
                ("YouTube".to_owned(), InternalPages::youtube_portal())
            }
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
