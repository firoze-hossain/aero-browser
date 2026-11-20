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
            Self::get_video_by_id("yt_humble"),
            Self::get_video_by_id("yt_mashrafe"),
            Self::get_video_by_id("yt_noman2"),
            Self::get_video_by_id("yt_parliament"),
            Self::get_video_by_id("yt_barsha"),
            Self::get_video_by_id("4NlKj7Saogc"),
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
            "yt_humble" => YouTubeVideo {
                video_id: "yt_humble".to_owned(),
                title: "Chupchap Shobaike Chere Dite Shikhun | Powerful Motivational Speech | Motiversity".to_owned(),
                channel: "Motiversity Bangla".to_owned(),
                views: "143K views".to_owned(),
                duration: "18:26".to_owned(),
                published: "3w ago".to_owned(),
                description: "Powerful motivational speech on self-respect, inner peace, and letting go of negative surroundings.".to_owned(),
                verified: true,
            },
            "yt_mashrafe" => YouTubeVideo {
                video_id: "yt_mashrafe".to_owned(),
                title: "Boyosko Chehara Niye Deshe Ferar Akuti Mashrafe Bin Mortaza | Celebrity News BD".to_owned(),
                channel: "Celebrity News BD".to_owned(),
                views: "2M views".to_owned(),
                duration: "2:09".to_owned(),
                published: "1d ago".to_owned(),
                description: "Mashrafe Mortaza opens up about his journey, physical fitness, and heartfelt emotional message to the country.".to_owned(),
                verified: true,
            },
            "yt_noman2" => YouTubeVideo {
                video_id: "yt_noman2".to_owned(),
                title: "Asokti: Nofske Niyontron Korar Upay | Dr. Nouman Ali Khan Bangla Dubbing".to_owned(),
                channel: "Al Quran & Hidayat".to_owned(),
                views: "186K views".to_owned(),
                duration: "19:15".to_owned(),
                published: "7mo ago".to_owned(),
                description: "Dr. Nouman Ali Khan provides powerful Quranic guidance on overcoming desires, discipline, and spiritual growth.".to_owned(),
                verified: true,
            },
            "yt_parliament" => YouTubeVideo {
                video_id: "yt_parliament".to_owned(),
                title: "Mugdho Puro Sangsad (Historic Parliament Debate) | ATN News".to_owned(),
                channel: "ATN News".to_owned(),
                views: "1.2M views".to_owned(),
                duration: "22:34".to_owned(),
                published: "2d ago".to_owned(),
                description: "Historic parliament address and floor speeches broadcast live from Jatiya Sangsad Bhaban.".to_owned(),
                verified: true,
            },
            "yt_barsha" => YouTubeVideo {
                video_id: "yt_barsha".to_owned(),
                title: "Barita Aji Chere Dite Hobe || Investigation".to_owned(),
                channel: "ATN News".to_owned(),
                views: "106K views".to_owned(),
                duration: "1:14".to_owned(),
                published: "23h ago".to_owned(),
                description: "Special investigative report by ATN News Investigation Unit.".to_owned(),
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
            "bY1cozLiK14" => YouTubeVideo {
                video_id: "bY1cozLiK14".to_owned(),
                title: "Vlad and Niki explore new ice cream truck adventure (Kids Fun)".to_owned(),
                channel: "Niki TV".to_owned(),
                views: "11M views".to_owned(),
                duration: "4:47".to_owned(),
                published: "5y ago".to_owned(),
                description: "Vlad and Niki explore new ice cream truck adventure for kids with giant gelato cones!".to_owned(),
                verified: true,
            },
            "pagi23" => YouTubeVideo {
                video_id: "pagi23".to_owned(),
                title: "Pagol Naki Song (Original) | WAQAS Official Lyric Video".to_owned(),
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
                title: "Building NovaOS 2.0: Writing an Operating System in Pure Rust".to_owned(),
                channel: "NovaOS Engineering".to_owned(),
                views: "542K views".to_owned(),
                duration: "42:15".to_owned(),
                published: "3d ago".to_owned(),
                description: "Deep dive into building NovaOS 2.0 kernel, nova_uring async rings, and Aero browser engine in pure Rust.".to_owned(),
                verified: true,
            },
            "rel_marriage" => YouTubeVideo {
                video_id: "rel_marriage".to_owned(),
                title: "Biyer Purbe Kichu Proyojonio Poramosho • Nouman Ali Khan".to_owned(),
                channel: "Bangla Waz".to_owned(),
                views: "304K views".to_owned(),
                duration: "30:32".to_owned(),
                published: "1mo ago".to_owned(),
                description: "Essential pre-marital advice and Islamic wisdom for youth by Dr. Nouman Ali Khan.".to_owned(),
                verified: true,
            },
            "rel_hajj" => YouTubeVideo {
                video_id: "rel_hajj".to_owned(),
                title: "Hojjer Pothe Nobiji Part 4 || Nouman Ali Khan".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "19K views".to_owned(),
                duration: "45:59".to_owned(),
                published: "1y ago".to_owned(),
                description: "The sacred journey of the Prophet Muhammad (PBUH) on Hajj.".to_owned(),
                verified: true,
            },
            "rel_sirah" => YouTubeVideo {
                video_id: "rel_sirah".to_owned(),
                title: "Sobar Jibonei Sukher Obhinoy Keno Eto Mikhut? - Nouman Ali Khan".to_owned(),
                channel: "Sirah".to_owned(),
                views: "1.3K views".to_owned(),
                duration: "38:57".to_owned(),
                published: "1d ago".to_owned(),
                description: "Why does everyone pretend to be happy? Profound reflections by Dr. Nouman Ali Khan.".to_owned(),
                verified: true,
            },
            "rel_destroy" => YouTubeVideo {
                video_id: "rel_destroy".to_owned(),
                title: "Amra Nijekey Dhongsho Kore Felchi? | Nouman Ali Khan Bangla".to_owned(),
                channel: "Islam Daily".to_owned(),
                views: "2.3K views".to_owned(),
                duration: "24:10".to_owned(),
                published: "1y ago".to_owned(),
                description: "Are we destroying our own spiritual well-being? Islamic reminders.".to_owned(),
                verified: true,
            },
            _ => YouTubeVideo {
                video_id: vid.to_owned(),
                title: format!("Featured Video: {}", vid),
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
        if v.title.contains("Chupchap") || v.title.contains("চুপচাপ") || v.video_id == "yt_humble" {
            (
                "#18181b",
                "#3f3f46",
                "#09090b",
                "#f59e0b",
                "🔥 MOTIVATIONAL SPEECH",
                "CHUPCHAP SORE JAN",
                "Powerful Self-Respect & Growth • Motiversity",
                "#1c1917",
                "MB",
            )
        } else if v.title.contains("Mashrafe") || v.title.contains("মাশরাফি") || v.video_id == "yt_mashrafe" {
            (
                "#14532d",
                "#16a34a",
                "#052e16",
                "#86efac",
                "🏏 CRICKET EXCLUSIVE",
                "MASHRAFE BIN MORTAZA",
                "Amar Boyos Hoyeche Deshe Ashte Den",
                "#dc2626",
                "CN",
            )
        } else if v.title.contains("Asokti") || v.title.contains("আসক্তি") || v.video_id == "yt_noman2" {
            (
                "#064e3b",
                "#059669",
                "#022c22",
                "#34d399",
                "🕌 ISLAMIC GUIDANCE",
                "ASOKTI: NOFS NIYONTRON",
                "Nouman Ali Khan • Quranic Spiritual Focus",
                "#78350f",
                "AQ",
            )
        } else if v.title.contains("Vlad") || v.channel.contains("Niki") {
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
        } else if v.title.contains("Parliament") || v.video_id == "yt_parliament" {
            (
                "#4c0519",
                "#e11d48",
                "#0f172a",
                "#fda4af",
                "📺 NATIONAL PARLIAMENT",
                "MUGDHO PURO SANGSAD",
                "ATN News Prime Time Report",
                "#dc2626",
                "ATN",
            )
        } else if v.title.contains("Barita") || v.video_id == "yt_barsha" {
            (
                "#713f12",
                "#eab308",
                "#1c1917",
                "#fde047",
                "📰 SPECIAL INVESTIGATION",
                "BARITA AJI CHERE DITE HOBE",
                "Entertainment News Investigation",
                "#dc2626",
                "ATN",
            )
        } else {
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
        &'static str, // stage_bg
        &'static str, // stage_border
        &'static str, // hud_tag
        &'static str, // watermark
        &'static str, // backdrop
        String,       // headline
        String,       // sub
        &'static str, // chyron_badge
        String,       // chyron_text
        &'static str, // avatar_bg
        &'static str, // avatar_text
    ) {
        if v.video_id == "yt_parliament" || v.title.contains("Parliament") {
            (
                "#020617",
                "#881337",
                "1080p60 HD • JATIYA SANGSAD BHABAN",
                "ATN NEWS 24 HD",
                "🏛️ NATIONAL PARLIAMENT OF BANGLADESH",
                "MUGDHO PURO SANGSAD • PARLIAMENTARY DEBATE".to_owned(),
                "Historic National Assembly Session • Prime Minister & All Benches Present".to_owned(),
                "⚡ ATN NEWS SPECIAL REPORT",
                "Special national reform bill passed unanimously across all parliamentary benches".to_owned(),
                "#dc2626",
                "ATN",
            )
        } else if v.video_id == "yt_mashrafe" || v.title.contains("মাশরাফি") || v.title.contains("Mashrafe") {
            (
                "#052e16",
                "#15803d",
                "1080p60 HD • ATN SPORTS SPECIAL",
                "CELEBRITY NEWS BD",
                "🏏 BANGLADESH CRICKET HEADQUARTERS",
                "MASHRAFE BIN MORTAZA • PRESS INTERVIEW".to_owned(),
                "\"Amar Boyos Hoyeche Amake Deshe Ashte Den\" • Live Press Conference".to_owned(),
                "🔴 CRICKET EXCLUSIVE",
                "Former captain Mashrafe shares heartfelt emotional message on national duty.".to_owned(),
                "#dc2626",
                "CN",
            )
        } else if v.video_id == "yt_humble" || v.title.contains("চুপচাপ") || v.title.contains("Chupchap") {
            (
                "#09090b",
                "#d97706",
                "1080p60 HI-FI • MOTIVERSITY BANGLA",
                "MOTIVERSITY",
                "🔥 SILENT RESOLVE & INNER STRENGTH",
                "CHUPCHAP SORE JAN • POWERFUL SPEECH".to_owned(),
                "Learn to quietly let go of negative surroundings and focus on growth".to_owned(),
                "⚡ LIFE LESSONS",
                "\"Never argue with ignorance. Step aside quietly and let your results speak.\"".to_owned(),
                "#1c1917",
                "MB",
            )
        } else if v.video_id == "bY1cozLiK14" || v.title.contains("Vlad") {
            (
                "#082f49",
                "#0284c7",
                "4K UHD 60FPS • NIKI TV OFFICIAL",
                "VLAD & NIKI",
                "🍦 MEGA ICE CREAM TRUCK EXPEDITION",
                "VLAD AND NIKI 4K GELATO QUEST".to_owned(),
                "Exploring colorful flavors with brother Niki and family".to_owned(),
                "🌟 KIDS ADVENTURE",
                "\"Look at this giant gelato cone! We found all five rainbow flavors!\"".to_owned(),
                "#0284c7",
                "VLAD",
            )
        } else if v.video_id == "pagi23" || v.title.contains("Pagol Naki") {
            (
                "#3b0764",
                "#9333ea",
                "1080p HI-FI • ACOUSTIC SESSIONS",
                "WAQAS OFFICIAL",
                "🎸 STUDIO UNPLUGGED LIVE",
                "PAGOL NAKI (ORIGINAL) • ACOUSTIC".to_owned(),
                "Bengali Diaries Volume 1 • Studio Sound Engineering Master".to_owned(),
                "🎵 STUDIO MASTER",
                "\"Tumi ki pagol naki... amar moner kotha shune jao ekbar...\"".to_owned(),
                "#a855f7",
                "WAQAS",
            )
        } else if v.video_id == "XsNGS_bxzwg" || v.title.contains("NovaOS") {
            (
                "#0f172a",
                "#2563eb",
                "1080p60 HD • NOVAOS SYSTEMS LAB",
                "RUST KERNEL ARCHITECTURE",
                "⚡ BUILDING NOVAOS 2.0 FROM SCRATCH",
                "WRITING AN OPERATING SYSTEM IN PURE RUST".to_owned(),
                "Zero C, zero inline ASM crates, monolithic microkernel & nova_uring".to_owned(),
                "🦀 SYSTEMS ENGINEERING",
                "\"Every single line of NovaOS, window manager and Aero browser is pure Rust.\"".to_owned(),
                "#ff5500",
                "NOVA",
            )
        } else if v.video_id == "rel_marriage" || v.title.contains("Biyer Purbe") {
            (
                "#022c22",
                "#059669",
                "1080p60 HD • BAYYINAH BANGLA",
                "BANGLA WAZ",
                "📖 ISLAMIC PRE-MARITAL COUNSELING",
                "BIYER PURBE KICHU PROYOJONIO PORAMOSHO".to_owned(),
                "Dr. Nouman Ali Khan • Advice for Muslim Youth".to_owned(),
                "💍 MATRIMONIAL GUIDANCE",
                "\"Marriage requires immense patience, emotional maturity and mutual respect.\"".to_owned(),
                "#059669",
                "NAK",
            )
        } else {
            (
                "#022c22",
                "#059669",
                "1080p60 HD • BAYYINAH AUDITORIUM",
                "AL QURAN & HIDAYAT",
                "🕌 BAYYINAH LECTURE SERIES",
                v.title.clone(),
                "Dr. Nouman Ali Khan • Advice for Muslim Youth: Purpose & Identity".to_owned(),
                "📖 SPIRITUAL WISDOM",
                "\"Our true strength is building moral clarity, patience and spiritual resilience.\"".to_owned(),
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
    padding: 8px 24px;
    display: block;
    width: 100%;
    box-sizing: border-box;
}}
.hamburger {{
    display: inline-block;
    font-size: 20px;
    color: #0f0f0f;
    width: 28px;
    cursor: pointer;
    vertical-align: middle;
}}
.logo {{
    display: inline-block;
    font-size: 19px;
    font-weight: bold;
    color: #0f0f0f;
    width: 140px;
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
    width: 580px;
    margin-left: 12%;
    vertical-align: middle;
}}
.search-pill {{
    background-color: #ffffff;
    border-right: none;
    border-radius: 20px 0 0 20px;
    padding: 9px 18px;
    width: 440px;
    color: #0f0f0f;
    font-size: 14px;
    display: inline-block;
    vertical-align: middle;
}}
.search-btn {{
    background-color: #f8f8f8;
    border: 1px solid #cccccc;
    border-radius: 0 20px 20px 0;
    width: 60px;
    padding: 9px 0;
    text-align: center;
    color: #0f0f0f;
    font-size: 15px;
    display: inline-block;
    vertical-align: middle;
    cursor: pointer;
}}
.mic-btn {{
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    border-radius: 50%;
    width: 38px;
    height: 38px;
    display: inline-block;
    margin-left: 12px;
    text-align: center;
    line-height: 38px;
    color: #0f0f0f;
    font-size: 15px;
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
    padding: 8px 18px;
    font-size: 13px;
    font-weight: bold;
    margin-top: 2px;
    margin-right: 16px;
    cursor: pointer;
}}
.sugg-dropdown {{
    position: absolute;
    top: 52px;
    left: 260px;
    width: 460px;
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
    width: 100%;
}}
.sidebar {{
    display: inline-block;
    width: 17%;
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
    width: 82%;
    vertical-align: top;
    padding: 36px 24px;
    box-sizing: border-box;
}}
.welcome-card {{
    background-color: #f9f9f9;
    border: 1px solid #e5e5e5;
    border-radius: 16px;
    padding: 42px 28px;
    text-align: center;
    max-width: 580px;
    margin: 16px auto 26px auto;
}}
.welcome-title {{
    font-size: 22px;
    font-weight: bold;
    color: #0f0f0f;
    margin-bottom: 10px;
}}
.welcome-sub {{
    font-size: 14px;
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
    padding: 8px 18px;
    border-radius: 18px;
    font-size: 13px;
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
                        <div class=\"thumb-top\">
                            <span class=\"thumb-tag\" style=\"background-color:{}; color:{};\">{}</span>
                            <span class=\"dur-badge\">{}</span>
                        </div>
                        <div class=\"thumb-headline\">{}</div>
                        <div class=\"thumb-subtext\">{}</div>
                    </div>
                    <div class=\"result-info\">
                        <div class=\"res-title\">{}</div>
                        <div class=\"res-meta\">{} • {}</div>
                        <div class=\"res-channel\"><span class=\"ch-avatar\" style=\"background-color:{}; color:#ffffff;\">{}</span> {} <span class=\"verified\">✔</span></div>
                        <div class=\"res-desc\">{}</div>
                    </div>
                </div>",
                bg_color, border_color, badge_bg, badge_color, tag, v.duration, headline, subtext,
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
    padding: 8px 24px;
    display: block;
    width: 100%;
    box-sizing: border-box;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #0f0f0f; width: 140px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 580px; margin-left: 12%; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 9px 18px; width: 440px; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 60px; padding: 9px 0; text-align: center; color: #0f0f0f; font-size: 15px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 38px; height: 38px; display: inline-block; margin-left: 12px; text-align: center; line-height: 38px; color: #0f0f0f; font-size: 15px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 8px 18px; font-size: 13px; font-weight: bold; margin-top: 2px; margin-right: 16px; cursor: pointer; }}
.layout-row {{ display: block; width: 100%; }}
.sidebar {{ display: inline-block; width: 17%; background-color: #ffffff; border-right: 1px solid #e5e5e5; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #0f0f0f; border-radius: 10px; margin: 2px 10px; cursor: pointer; }}
.side-active {{ background-color: #f2f2f2; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #e5e5e5; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 82%; vertical-align: top; padding: 16px 24px; box-sizing: border-box; }}
.result-card {{ display: block; margin-bottom: 24px; cursor: pointer; }}
.thumb-widescreen {{ display: inline-block; width: 320px; height: 180px; border-radius: 12px; vertical-align: top; padding: 10px 12px; box-sizing: border-box; }}
.thumb-top {{ display: block; width: 100%; }}
.thumb-tag {{ font-size: 10px; font-weight: bold; padding: 4px 8px; border-radius: 6px; display: inline-block; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 3px 6px; font-size: 11px; font-weight: bold; float: right; display: inline-block; }}
.thumb-headline {{ color: #ffffff; font-size: 15px; font-weight: bold; text-align: center; line-height: 1.4; margin-top: 32px; }}
.thumb-subtext {{ color: #cbd5e1; font-size: 11px; text-align: center; margin-top: 8px; }}
.result-info {{ display: inline-block; width: 56%; padding-left: 18px; vertical-align: top; }}
.res-title {{ font-size: 16px; font-weight: bold; color: #0f0f0f; line-height: 1.35; margin-bottom: 6px; }}
.res-meta {{ font-size: 12px; color: #606060; margin-bottom: 6px; }}
.res-channel {{ font-size: 12px; color: #606060; margin-bottom: 8px; }}
.ch-avatar {{ border-radius: 50%; padding: 3px 7px; margin-right: 6px; font-weight: bold; font-size: 10px; }}
.verified {{ color: #606060; font-size: 11px; }}
.res-desc {{ font-size: 12px; color: #606060; line-height: 1.45; }}
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

    /// Builds the authentic Cinema Player watch page matching Image 3 in White Mode.
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
        let (stage_bg, stage_border, hud_tag, watermark, backdrop, headline, sub, chyron_badge, chyron_text, avatar_bg, avatar_text) =
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
            let (rbg, rborder, _, _, rtag, rhead, _, _, _) = Self::get_video_thumbnail_details(r);
            rel_cards.push_str(&format!(
                "<div class=\"rel-item\">
                    <div class=\"rel-thumb\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"dur-badge\">{}</div>
                        <div style=\"color:#ffffff; font-size:9px; font-weight:bold; padding:4px 6px;\">{}</div>
                        <div style=\"color:#e2e8f0; font-size:11px; font-weight:bold; text-align:center; padding-top:10px;\">▶ HD</div>
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
    padding: 8px 24px;
    display: block;
    width: 100%;
    box-sizing: border-box;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #0f0f0f; width: 140px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 580px; margin-left: 12%; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 9px 18px; width: 440px; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 60px; padding: 9px 0; text-align: center; color: #0f0f0f; font-size: 15px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 38px; height: 38px; display: inline-block; margin-left: 12px; text-align: center; line-height: 38px; color: #0f0f0f; font-size: 15px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 8px 18px; font-size: 13px; font-weight: bold; margin-top: 2px; margin-right: 16px; cursor: pointer; }}
.watch-container {{ padding: 16px 24px; display: block; width: 100%; box-sizing: border-box; }}
.player-column {{ display: inline-block; width: 68%; vertical-align: top; }}
.cinema-player {{
    background-color: #000000;
    border-radius: 12px;
    border: 1px solid #1e293b;
    display: block;
    width: 100%;
}}
.video-broadcast-frame {{
    height: 380px;
    border-radius: 12px 12px 0 0;
    position: relative;
    padding: 14px 18px;
    box-sizing: border-box;
    display: block;
}}
.broadcast-top-bar {{ display: block; margin-bottom: 24px; }}
.live-pill {{ background-color: #dc2626; color: #ffffff; border-radius: 4px; padding: 2px 7px; font-size: 11px; font-weight: bold; margin-right: 10px; }}
.hud-stream-tag {{ color: #f8fafc; font-size: 12px; font-weight: bold; }}
.network-watermark {{ float: right; color: #94a3b8; font-size: 11px; font-weight: bold; }}
.broadcast-main-scene {{ text-align: center; padding-top: 40px; display: block; }}
.scene-backdrop {{ font-size: 22px; color: #38bdf8; font-weight: bold; margin-bottom: 8px; }}
.scene-headline {{ font-size: 20px; color: #ffffff; font-weight: bold; line-height: 1.35; margin-bottom: 6px; }}
.scene-sub {{ font-size: 13px; color: #cbd5e1; }}
.broadcast-lower-third {{
    position: absolute;
    bottom: 38px;
    left: 18px;
    right: 18px;
    background-color: rgba(15, 23, 42, 0.92);
    border-left: 4px solid #ef4444;
    border-radius: 6px;
    padding: 8px 14px;
    display: block;
}}
.chyron-badge {{ color: #ef4444; font-size: 10px; font-weight: bold; margin-bottom: 2px; }}
.chyron-text {{ color: #ffffff; font-size: 13px; font-weight: bold; line-height: 1.3; }}
.broadcast-spectrum-bar {{
    position: absolute;
    bottom: 10px;
    left: 18px;
    right: 18px;
    font-size: 11px;
    color: #38bdf8;
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
    padding: 8px 16px;
    font-size: 12px;
    color: #f1f1f1;
    border-radius: 0 0 12px 12px;
    display: block;
}}
.ctrl-item {{ display: inline-block; margin-right: 14px; cursor: pointer; }}
.float-r {{ float: right; margin-left: 12px; margin-right: 0; }}
.shield-badge {{ display: inline-block; color: #ff5500; font-weight: bold; font-size: 11px; margin-left: 8px; }}
.vid-heading {{ font-size: 18px; font-weight: bold; color: #0f0f0f; margin-top: 14px; line-height: 1.35; }}
.channel-row {{ margin-top: 14px; padding-bottom: 14px; border-bottom: 1px solid #e5e5e5; display: block; }}
.ch-avatar-big {{
    border-radius: 50%;
    width: 40px;
    height: 40px;
    display: inline-block;
    text-align: center;
    line-height: 40px;
    font-weight: bold;
    font-size: 12px;
    vertical-align: middle;
}}
.ch-details {{ display: inline-block; vertical-align: middle; margin-left: 12px; width: 220px; }}
.ch-name {{ font-size: 14px; font-weight: bold; color: #0f0f0f; }}
.ch-subs {{ font-size: 12px; color: #606060; }}
.sub-pill {{
    display: inline-block;
    background-color: {};
    color: {};
    font-weight: bold;
    padding: 8px 18px;
    border-radius: 18px;
    font-size: 13px;
    vertical-align: middle;
    margin-left: 12px;
    border: 1px solid #cccccc;
    cursor: pointer;
}}
.action-pills {{ display: inline-block; float: right; vertical-align: middle; margin-top: 4px; }}
.action-pill {{
    display: inline-block;
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    border-radius: 18px;
    padding: 7px 14px;
    font-size: 12px;
    color: #0f0f0f;
    margin-left: 6px;
    cursor: pointer;
}}
.desc-box {{
    background-color: #f2f2f2;
    border-radius: 12px;
    padding: 14px 18px;
    margin-top: 14px;
    font-size: 13px;
    color: #0f0f0f;
    line-height: 1.5;
}}
.related-column {{
    display: inline-block;
    width: 30%;
    vertical-align: top;
    margin-left: 1%;
}}
.rel-header {{ font-size: 15px; font-weight: bold; color: #0f0f0f; margin-bottom: 12px; }}
.rel-item {{ display: block; margin-bottom: 14px; cursor: pointer; }}
.rel-thumb {{ display: inline-block; width: 140px; height: 80px; border-radius: 8px; vertical-align: top; position: relative; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 2px 4px; font-size: 10px; font-weight: bold; float: right; margin: 4px; }}
.rel-info {{ display: inline-block; width: 55%; padding-left: 10px; vertical-align: top; }}
.rel-title {{ font-size: 13px; font-weight: bold; color: #0f0f0f; line-height: 1.3; margin-bottom: 4px; }}
.rel-channel {{ font-size: 11px; color: #606060; margin-bottom: 2px; }}
.rel-meta {{ font-size: 11px; color: #606060; }}
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
            <div class=\"video-broadcast-frame\" style=\"background-color:{}; border:1px solid {};\">
                <div class=\"broadcast-top-bar\">
                    <span class=\"live-pill\">🔴 LIVE</span>
                    <span class=\"hud-stream-tag\">{}</span>
                    <span class=\"network-watermark\">{}</span>
                </div>
                <div class=\"broadcast-main-scene\">
                    <div class=\"scene-backdrop\">{}</div>
                    <div class=\"scene-headline\">{}</div>
                    <div class=\"scene-sub\">{}</div>
                </div>
                <div class=\"broadcast-lower-third\">
                    <div class=\"chyron-badge\">{}</div>
                    <div class=\"chyron-text\">{}</div>
                </div>
                <div class=\"broadcast-spectrum-bar\">
                    {}
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
            stage_bg,
            stage_border,
            hud_tag,
            watermark,
            backdrop,
            headline,
            sub,
            chyron_badge,
            chyron_text,
            spectrum_str,
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

    /// Builds the rich Home Feed matching Image 1 in White Mode.
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

        // Split videos into Row 1 (first 3) and Row 2 (next 3)
        let row1_videos = if videos.len() >= 3 { &videos[0..3] } else { videos };
        let row2_videos = if videos.len() >= 6 { &videos[3..6] } else if videos.len() > 3 { &videos[3..] } else { &[] };

        let get_feed_card_lines = |vid: &str| -> (&'static str, &'static str) {
            match vid {
                "yt_humble" => ("Chupchap Shobaike", "Chere Dite Shikhun"),
                "yt_mashrafe" => ("Mashrafe Bin Mortaza", "Deshe Ferar Akuti"),
                "yt_noman2" => ("Asokti: Nofs Niyontron", "Dr. Nouman Ali Khan"),
                "yt_barsha" => ("Barita Aji Chere Dite", "Hobe | Investigation"),
                "yt_parliament" => ("Mugdho Puro Sangsad", "Historic Floor Debate"),
                _ => ("Muslim Torunder", "Uddesshe Kichu Kotha"),
            }
        };

        let mut row1_html = String::new();
        for v in row1_videos {
            let (line1, line2) = get_feed_card_lines(&v.video_id);
            let (bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, avatar_bg, avatar_text) =
                Self::get_video_thumbnail_details(v);

            row1_html.push_str(&format!(
                "<div class=\"grid-card\">
                    <div class=\"card-thumb\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"thumb-top\">
                            <span class=\"thumb-tag\" style=\"background-color:{}; color:{};\">{}</span>
                            <span class=\"dur-badge\">{}</span>
                        </div>
                        <div class=\"thumb-headline\">{}</div>
                        <div class=\"thumb-subtext\">{}</div>
                    </div>
                    <div class=\"card-body\">
                        <div class=\"card-avatar\" style=\"background-color:{}; color:#ffffff;\">{}</div>
                        <div class=\"card-text\">
                            <div class=\"card-title\">{}</div>
                            <div class=\"card-title-sub\">{}</div>
                            <div class=\"card-channel\">{} ✔</div>
                            <div class=\"card-meta\">{} • {}</div>
                        </div>
                    </div>
                </div>",
                bg_color, border_color, badge_bg, badge_color, tag, v.duration, headline, subtext,
                avatar_bg, avatar_text, line1, line2, v.channel, v.views, v.published
            ));
        }

        let mut row2_html = String::new();
        for v in row2_videos {
            let (line1, line2) = get_feed_card_lines(&v.video_id);
            let (bg_color, border_color, badge_bg, badge_color, tag, headline, subtext, avatar_bg, avatar_text) =
                Self::get_video_thumbnail_details(v);

            row2_html.push_str(&format!(
                "<div class=\"grid-card\">
                    <div class=\"card-thumb\" style=\"background-color:{}; border:1px solid {};\">
                        <div class=\"thumb-top\">
                            <span class=\"thumb-tag\" style=\"background-color:{}; color:{};\">{}</span>
                            <span class=\"dur-badge\">{}</span>
                        </div>
                        <div class=\"thumb-headline\">{}</div>
                        <div class=\"thumb-subtext\">{}</div>
                    </div>
                    <div class=\"card-body\">
                        <div class=\"card-avatar\" style=\"background-color:{}; color:#ffffff;\">{}</div>
                        <div class=\"card-text\">
                            <div class=\"card-title\">{}</div>
                            <div class=\"card-title-sub\">{}</div>
                            <div class=\"card-channel\">{} ✔</div>
                            <div class=\"card-meta\">{} • {}</div>
                        </div>
                    </div>
                </div>",
                bg_color, border_color, badge_bg, badge_color, tag, v.duration, headline, subtext,
                avatar_bg, avatar_text, line1, line2, v.channel, v.views, v.published
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
    padding: 8px 24px;
    display: block;
    width: 100%;
    box-sizing: border-box;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #0f0f0f; width: 28px; cursor: pointer; vertical-align: middle; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #0f0f0f; width: 140px; cursor: pointer; vertical-align: middle; }}
.logo-badge {{ font-size: 10px; color: #606060; margin-left: 2px; }}
.search-container {{ display: inline-block; width: 580px; margin-left: 12%; vertical-align: middle; }}
.search-pill {{ background-color: #ffffff; border-right: none; border-radius: 20px 0 0 20px; padding: 9px 18px; width: 440px; color: #0f0f0f; font-size: 14px; display: inline-block; vertical-align: middle; }}
.search-btn {{ background-color: #f8f8f8; border: 1px solid #cccccc; border-radius: 0 20px 20px 0; width: 60px; padding: 9px 0; text-align: center; color: #0f0f0f; font-size: 15px; display: inline-block; vertical-align: middle; cursor: pointer; }}
.mic-btn {{ background-color: #f2f2f2; border: 1px solid #e5e5e5; border-radius: 50%; width: 38px; height: 38px; display: inline-block; margin-left: 12px; text-align: center; line-height: 38px; color: #0f0f0f; font-size: 15px; vertical-align: middle; cursor: pointer; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #065fd4; color: #065fd4; background-color: #ffffff; border-radius: 18px; padding: 8px 18px; font-size: 13px; font-weight: bold; margin-top: 2px; margin-right: 16px; cursor: pointer; }}
.layout-row {{ display: block; width: 100%; }}
.sidebar {{ display: inline-block; width: 17%; background-color: #ffffff; border-right: 1px solid #e5e5e5; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #0f0f0f; border-radius: 10px; margin: 2px 10px; cursor: pointer; }}
.side-active {{ background-color: #f2f2f2; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #e5e5e5; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 82%; vertical-align: top; padding: 16px 1%; box-sizing: border-box; }}
.chips-bar {{ margin-bottom: 20px; display: block; width: 100%; }}
.chip {{
    display: inline-block;
    background-color: #f2f2f2;
    border: 1px solid #e5e5e5;
    color: #0f0f0f;
    padding: 7px 16px;
    border-radius: 16px;
    font-size: 13px;
    margin-right: 6px;
    margin-bottom: 6px;
    cursor: pointer;
}}
.chip-active {{ background-color: #0f0f0f; color: #ffffff; font-weight: bold; }}
.grid-row {{ display: block; width: 100%; margin-bottom: 16px; }}
.grid-card {{
    display: inline-block;
    width: 31%;
    margin: 6px 1% 20px 1%;
    vertical-align: top;
    cursor: pointer;
}}
.card-thumb {{
    border-radius: 12px;
    height: 180px;
    padding: 10px 12px;
    box-sizing: border-box;
    display: block;
}}
.thumb-top {{ display: block; width: 100%; }}
.thumb-tag {{ font-size: 10px; font-weight: bold; padding: 4px 8px; border-radius: 6px; display: inline-block; }}
.dur-badge {{ background-color: rgba(0, 0, 0, 0.85); color: #ffffff; border-radius: 4px; padding: 3px 6px; font-size: 11px; font-weight: bold; float: right; display: inline-block; }}
.thumb-headline {{ color: #ffffff; font-size: 15px; font-weight: bold; text-align: center; line-height: 1.4; margin-top: 32px; }}
.thumb-subtext {{ color: #cbd5e1; font-size: 11px; text-align: center; margin-top: 8px; }}
.card-body {{ margin-top: 10px; display: block; }}
.card-avatar {{
    border-radius: 50%;
    width: 36px;
    height: 36px;
    display: inline-block;
    text-align: center;
    line-height: 36px;
    font-size: 11px;
    font-weight: bold;
    vertical-align: top;
    margin-right: 12px;
}}
.card-text {{ display: inline-block; width: 75%; vertical-align: top; }}
.card-title {{ font-size: 14px; font-weight: bold; color: #0f0f0f; line-height: 1.25; margin-bottom: 2px; }}
.card-title-sub {{ font-size: 14px; font-weight: bold; color: #0f0f0f; line-height: 1.25; margin-bottom: 6px; }}
.card-channel {{ font-size: 12px; color: #606060; margin-bottom: 2px; }}
.card-meta {{ font-size: 11px; color: #606060; }}
.shorts-section {{ margin-top: 10px; margin-bottom: 26px; display: block; width: 100%; border-top: 1px solid #e5e5e5; padding-top: 18px; }}
.shorts-header {{ font-size: 17px; font-weight: bold; color: #0f0f0f; margin-bottom: 14px; margin-left: 1%; }}
.shorts-grid {{ display: block; width: 100%; }}
.short-card {{
    display: inline-block;
    width: 23%;
    height: 250px;
    border-radius: 12px;
    margin: 4px 1%;
    padding: 10px;
    box-sizing: border-box;
    vertical-align: top;
    position: relative;
    cursor: pointer;
}}
.short-badge {{ background-color: rgba(0,0,0,0.8); color: #ffffff; border-radius: 4px; padding: 2px 6px; font-size: 10px; font-weight: bold; display: inline-block; }}
.short-center {{ color: #ffffff; font-size: 14px; font-weight: bold; text-align: center; padding-top: 60px; line-height: 1.3; }}
.short-title {{ color: #ffffff; font-size: 12px; font-weight: bold; position: absolute; bottom: 28px; left: 10px; right: 10px; line-height: 1.3; }}
.short-views {{ color: #cbd5e1; font-size: 10px; position: absolute; bottom: 10px; left: 10px; }}
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
            <div class=\"chip\">Nouman Ali Khan</div>
            <div class=\"chip\">News</div>
            <div class=\"chip\">Podcasts</div>
            <div class=\"chip\">Music</div>
            <div class=\"chip\">Gaming</div>
            <div class=\"chip\">Live</div>
            <div class=\"chip\">Bangla</div>
            <div class=\"chip\">Coding</div>
        </div>

        <div class=\"grid-row\">
            {}
        </div>

        <div class=\"shorts-section\">
            <div class=\"shorts-header\"><span style=\"color:#ff0000; font-size:18px;\">⚡</span> <b>Shorts</b></div>
            <div class=\"shorts-grid\">
                <div class=\"short-card\" style=\"background-color:#1e293b; border:1px solid #334155;\">
                    <div class=\"short-badge\">NEW</div>
                    <div class=\"short-center\">📚 STUDY HACK</div>
                    <div class=\"short-title\">Habits of Toppers • Exam Routine</div>
                    <div class=\"short-views\">3.4M views</div>
                </div>
                <div class=\"short-card\" style=\"background-color:#4c0519; border:1px solid #881337;\">
                    <div class=\"short-badge\">VIRAL</div>
                    <div class=\"short-center\">✨ BEAUTY WITH BRAIN</div>
                    <div class=\"short-title\">Perfect example of Beauty with brain</div>
                    <div class=\"short-views\">18M views</div>
                </div>
                <div class=\"short-card\" style=\"background-color:#713f12; border:1px solid #a16207;\">
                    <div class=\"short-badge\">FUNNY</div>
                    <div class=\"short-center\">🐯 TIGER COSTUME</div>
                    <div class=\"short-title\">Backyard comedy adventure</div>
                    <div class=\"short-views\">246K views</div>
                </div>
                <div class=\"short-card\" style=\"background-color:#064e3b; border:1px solid #047857;\">
                    <div class=\"short-badge\">HOLY</div>
                    <div class=\"short-center\">🕌 MASJID NABAWI</div>
                    <div class=\"short-title\">Rain blessing at the Prophet Mosque</div>
                    <div class=\"short-views\">1.2M views</div>
                </div>
            </div>
        </div>

        <div class=\"grid-row\">
            {}
        </div>
    </div>
</div>
</body>
</html>",
            search_border, display_input, row1_html, row2_html
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
                v_part.split('&').next().unwrap_or("yt_parliament")
            } else {
                "yt_parliament"
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
        let q_input = if !query_input.is_empty() {
            query_input
        } else if trimmed.contains("q=") {
            trimmed.split("q=").nth(1).unwrap_or("").split('&').next().unwrap_or("")
        } else {
            ""
        };
        let sugg_open = trimmed.contains("sugg=true") || is_focused;
        let html = Self::build_youtube_clean_home_html(q_input, sugg_open, is_focused, cursor_visible);
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
