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
    fn get_default_home_feed() -> Vec<YouTubeVideo> {
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
    fn get_default_search_results(query: &str) -> Vec<YouTubeVideo> {
        vec![
            YouTubeVideo {
                video_id: "bY1cozLiK14".to_owned(),
                title: "Vlad and Niki new mother explore ice cream truck (ভ্লাদ এবং নিকি)".to_owned(),
                channel: "Niki TV".to_owned(),
                views: "11M views".to_owned(),
                duration: "4:47".to_owned(),
                published: "5y ago".to_owned(),
                description: "Vlad and Niki explore new ice cream truck adventure for kids.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "4NlKj7Saogc".to_owned(),
                title: "Muslim Torunder Uddesshe Kichu Kotha || Nouman Ali Khan (NAKInBangla)".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "155K views".to_owned(),
                duration: "56:28".to_owned(),
                published: "1y ago".to_owned(),
                description: "Support us in patreon: https://patreon.com/spektrummedia Advice for Muslim Youth by Nouman Ali Khan.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "pagi23".to_owned(),
                title: "Pagol Naki Song (Original) | পাগল নাকি | WAQAS (Official Lyric Video)".to_owned(),
                channel: "WAQAS".to_owned(),
                views: "348K views".to_owned(),
                duration: "2:38".to_owned(),
                published: "2w ago".to_owned(),
                description: "Pagol Naki song | Official Lyric Video by WAQAS.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "LWzPB04DhCg".to_owned(),
                title: "Vul Naki Prem | ভুল নাকি প্রেম | Arosh Khan | Bangla Natok 2026".to_owned(),
                channel: "G Series Bangla Natok".to_owned(),
                views: "3.9M views".to_owned(),
                duration: "44:59".to_owned(),
                published: "6mo ago".to_owned(),
                description: "Presenting Special Romantic Drama 'Vul Naki Prem' starring Arosh Khan.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "XsNGS_bxzwg".to_owned(),
                title: "Chakri Bachabo Naki Girlfriend (Full Natok) | চাকরি বাঁচাবো নাকি গার্লফ্রেন্ড".to_owned(),
                channel: "Eagle Premier Station".to_owned(),
                views: "1.1M views".to_owned(),
                duration: "49:05".to_owned(),
                published: "1y ago".to_owned(),
                description: "Eagle Team Presents New Comedy Drama 'Chakri Bachabo Naki Girlfriend Bachabo'.".to_owned(),
                verified: true,
            },
        ]
    }

    #[cfg(feature = "std")]
    fn get_related_videos_for(video_id: &str) -> Vec<YouTubeVideo> {
        vec![
            YouTubeVideo {
                video_id: "rel_hajj".to_owned(),
                title: "Hojjer Pothe Nobiji Part 4 || Nouman Ali Khan".to_owned(),
                channel: "NAKInBangla".to_owned(),
                views: "19K views".to_owned(),
                duration: "45:53".to_owned(),
                published: "1y ago".to_owned(),
                description: "The historical and spiritual journey of Hajj.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "rel_sirah".to_owned(),
                title: "Sobar Jibonei Sukher Obhinoy Keno Eto Nikhut? - Nouman Ali Khan".to_owned(),
                channel: "SiRah".to_owned(),
                views: "1.3K views".to_owned(),
                duration: "38:57".to_owned(),
                published: "1d ago".to_owned(),
                description: "Reflections on mental peace and inner contentment in modern life.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "rel_marriage".to_owned(),
                title: "Biyer Purbe Kichu Proyojonio Poramorsho • Nouman Ali Khan".to_owned(),
                channel: "Bangla Waz".to_owned(),
                views: "304K views".to_owned(),
                duration: "30:32".to_owned(),
                published: "11mo ago".to_owned(),
                description: "Crucial guidance and advice on marriage preparation.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "rel_destroy".to_owned(),
                title: "Amra Nijekey Dhongsho Kore Felchi? | Noman Ali Khan Bangla".to_owned(),
                channel: "Islam Daily".to_owned(),
                views: "2.3K views".to_owned(),
                duration: "24:10".to_owned(),
                published: "1y ago".to_owned(),
                description: "Are we destroying our own potential? Deep soul searching lecture.".to_owned(),
                verified: true,
            },
            YouTubeVideo {
                video_id: "rel_humble".to_owned(),
                title: "True Men of Knowledge are Extremely Humble || Dr. Noman Ali Khan".to_owned(),
                channel: "Motivation".to_owned(),
                views: "6.5K views".to_owned(),
                duration: "17:06".to_owned(),
                published: "2d ago".to_owned(),
                description: "The noble character and modesty of true scholars.".to_owned(),
                verified: true,
            },
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
            let html = Self::build_youtube_search_results_html(q, &videos);
            Some((format!("{} - YouTube", q), html))
        } else {
            if videos.is_empty() {
                videos = Self::get_default_home_feed();
            }
            let html = Self::build_youtube_feed_html("All", &videos);
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

    /// Builds the clean initial landing page matching Image 1 & 2.
    #[cfg(feature = "std")]
    pub fn build_youtube_clean_home_html(query_input: &str, suggestions_open: bool) -> String {
        let display_input = if query_input.is_empty() {
            if suggestions_open { "naki in bangl" } else { "Search" }
        } else {
            query_input
        };
        let sugg_display = if suggestions_open { "block" } else { "none" };

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>YouTube</title>
<style>
body {{
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #0f0f0f;
    border-bottom: 1px solid #272727;
    height: 48px;
    padding: 8px 18px;
    display: block;
}}
.hamburger {{
    display: inline-block;
    font-size: 20px;
    color: #f1f1f1;
    width: 32px;
    cursor: pointer;
}}
.logo {{
    display: inline-block;
    font-size: 19px;
    font-weight: bold;
    color: #ffffff;
    width: 140px;
    cursor: pointer;
}}
.logo-badge {{
    font-size: 10px;
    color: #aaaaaa;
    margin-left: 2px;
}}
.search-container {{
    display: inline-block;
    width: 520px;
    margin-left: 40px;
    position: relative;
}}
.search-pill {{
    background-color: #121212;
    border: 1px solid #303030;
    border-radius: 20px;
    padding: 7px 18px;
    width: 420px;
    color: #f1f1f1;
    font-size: 14px;
    display: inline-block;
}}
.search-btn {{
    background-color: #222222;
    border: 1px solid #303030;
    border-radius: 0 20px 20px 0;
    padding: 7px 18px;
    color: #f1f1f1;
    font-size: 14px;
    display: inline-block;
    margin-left: -5px;
}}
.mic-btn {{
    background-color: #181818;
    border-radius: 50%;
    width: 34px;
    height: 34px;
    display: inline-block;
    margin-left: 10px;
    text-align: center;
    color: #fff;
    font-size: 14px;
}}
.sugg-box {{
    background-color: #212121;
    border: 1px solid #303030;
    border-radius: 12px;
    padding: 6px 12px;
    width: 420px;
    margin-top: 6px;
    text-align: left;
    display: {};
}}
.sugg-item {{
    padding: 7px 10px;
    font-size: 13px;
    color: #f1f1f1;
    border-bottom: 1px solid #2a2a2a;
}}
.signin-btn {{
    display: inline-block;
    float: right;
    border: 1px solid #3ea6ff;
    color: #3ea6ff;
    border-radius: 18px;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: bold;
    margin-top: 2px;
}}
.layout-row {{
    display: block;
}}
.sidebar {{
    display: inline-block;
    width: 190px;
    background-color: #0f0f0f;
    border-right: 1px solid #222222;
    padding-top: 12px;
    vertical-align: top;
}}
.side-item {{
    padding: 10px 18px;
    font-size: 13px;
    color: #f1f1f1;
    border-radius: 10px;
    margin: 2px 10px;
}}
.side-active {{
    background-color: #272727;
    font-weight: bold;
}}
.side-divider {{
    border-top: 1px solid #222222;
    margin: 10px 14px;
}}
.signin-prompt {{
    padding: 12px 18px;
    font-size: 12px;
    color: #aaaaaa;
    line-height: 1.4;
}}
.content-area {{
    display: inline-block;
    width: 780px;
    vertical-align: top;
    padding: 40px 20px;
}}
.welcome-card {{
    background-color: #1a1a1a;
    border: 1px solid #272727;
    border-radius: 16px;
    padding: 38px 24px;
    text-align: center;
    max-width: 520px;
    margin: 20px auto 30px auto;
}}
.welcome-title {{
    font-size: 21px;
    font-weight: bold;
    color: #ffffff;
    margin-bottom: 10px;
}}
.welcome-sub {{
    font-size: 13px;
    color: #aaaaaa;
    line-height: 1.5;
}}
.quick-pills {{
    text-align: center;
    margin-top: 16px;
}}
.pill {{
    display: inline-block;
    background-color: #272727;
    border: 1px solid #383838;
    color: #f1f1f1;
    padding: 7px 16px;
    border-radius: 16px;
    font-size: 12px;
    margin: 6px;
}}
.pill-active {{
    background-color: #ffffff;
    color: #0f0f0f;
    font-weight: bold;
}}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\">[>] YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\">{}</div>
        <div class=\"search-btn\">[Q]</div>
        <div class=\"mic-btn\">[🎙]</div>
        <div class=\"sugg-box\">
            <div class=\"sugg-item\">🔍 <b>naki in bangla</b></div>
            <div class=\"sugg-item\">🔍 naki in bangla natok</div>
            <div class=\"sugg-item\">🔍 naki in bangla song</div>
            <div class=\"sugg-item\">🔍 naki in bangla waz</div>
        </div>
    </div>
    <div class=\"signin-btn\">[👤 Sign in]</div>
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
        <div class=\"signin-prompt\">Sign in to like videos,<br>comment, and subscribe.<br><span style=\"color:#3ea6ff; font-weight:bold;\">[Sign in]</span></div>
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
            <div class=\"pill pill-active\">🔍 naki in bangla</div>
            <div class=\"pill\">🔥 Explore Trending Feed</div>
            <div class=\"pill\">🎵 Popular Music</div>
            <div class=\"pill\">🎮 Gaming Live</div>
            <div class=\"pill\">💻 Tech & Coding</div>
        </div>
    </div>
</div>
</body>
</html>",
            sugg_display, display_input
        )
    }

    /// Builds the authentic search results page matching Image 3.
    #[cfg(feature = "std")]
    pub fn build_youtube_search_results_html(query: &str, videos: &[YouTubeVideo]) -> String {
        let mut cards = String::new();
        for v in videos {
            cards.push_str(&format!(
                "<div class=\"result-card\">
                    <div class=\"thumb-widescreen\">
                        <div class=\"dur-badge\">{}</div>
                        <div style=\"color:#ff0000; font-size:18px; font-weight:bold; padding-top:45px; text-align:center;\">[▶ Play]</div>
                    </div>
                    <div class=\"result-info\">
                        <div class=\"res-title\">{}</div>
                        <div class=\"res-meta\">{} • {}</div>
                        <div class=\"res-channel\"><span class=\"ch-avatar\">[o]</span> {} <span class=\"verified\">✔</span></div>
                        <div class=\"res-desc\">{}</div>
                    </div>
                </div>",
                v.duration, v.title, v.views, v.published, v.channel, v.description
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - YouTube</title>
<style>
body {{
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #0f0f0f;
    border-bottom: 1px solid #272727;
    height: 48px;
    padding: 8px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #f1f1f1; width: 32px; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #ffffff; width: 140px; }}
.logo-badge {{ font-size: 10px; color: #aaaaaa; }}
.search-container {{ display: inline-block; width: 520px; margin-left: 40px; }}
.search-pill {{ background-color: #121212; border: 1px solid #303030; border-radius: 20px; padding: 7px 18px; width: 420px; color: #f1f1f1; font-size: 14px; display: inline-block; }}
.search-btn {{ background-color: #222222; border: 1px solid #303030; border-radius: 0 20px 20px 0; padding: 7px 18px; color: #f1f1f1; font-size: 14px; display: inline-block; margin-left: -5px; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #3ea6ff; color: #3ea6ff; border-radius: 18px; padding: 6px 14px; font-size: 13px; font-weight: bold; margin-top: 2px; }}
.layout-row {{ display: block; }}
.sidebar {{ display: inline-block; width: 190px; background-color: #0f0f0f; border-right: 1px solid #222222; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #f1f1f1; border-radius: 10px; margin: 2px 10px; }}
.side-active {{ background-color: #272727; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #222222; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 800px; vertical-align: top; padding: 20px 24px; }}
.result-card {{ display: block; margin-bottom: 22px; }}
.thumb-widescreen {{ display: inline-block; width: 300px; height: 140px; background-color: #1c1c1c; border-radius: 10px; vertical-align: top; position: relative; }}
.dur-badge {{ background-color: #000000; color: #ffffff; border-radius: 4px; padding: 2px 6px; font-size: 11px; font-weight: bold; float: right; margin: 6px; }}
.result-info {{ display: inline-block; width: 470px; padding-left: 18px; vertical-align: top; }}
.res-title {{ font-size: 15px; font-weight: bold; color: #ffffff; line-height: 1.3; margin-bottom: 6px; }}
.res-meta {{ font-size: 12px; color: #aaaaaa; margin-bottom: 6px; }}
.res-channel {{ font-size: 12px; color: #aaaaaa; margin-bottom: 8px; }}
.ch-avatar {{ background-color: #333333; border-radius: 50%; padding: 2px 5px; margin-right: 4px; }}
.verified {{ color: #aaaaaa; font-size: 11px; }}
.res-desc {{ font-size: 11px; color: #717171; line-height: 1.4; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\">[>] YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\">{}</div>
        <div class=\"search-btn\">[Q]</div>
    </div>
    <div class=\"signin-btn\">[👤 Sign in]</div>
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
            query, query, cards
        )
    }

    /// Builds the authentic Cinema Player watch page matching Image 4.
    #[cfg(feature = "std")]
    pub fn build_youtube_watch_html(
        video: &YouTubeVideo,
        related: &[YouTubeVideo],
        is_playing: bool,
        is_subscribed: bool,
        likes: u32,
    ) -> String {
        let play_badge = if is_playing { "▶ PLAYING (Live Stream Active)" } else { "⏸ PAUSED" };
        let play_btn = if is_playing { "[⏸ Pause]" } else { "[▶ Play]" };
        let sub_btn = if is_subscribed { "[Subscribed ✓]" } else { "[Subscribe]" };
        let sub_color = if is_subscribed { "#333333" } else { "#ffffff" };
        let sub_text_color = if is_subscribed { "#aaaaaa" } else { "#0f0f0f" };

        let mut rel_cards = String::new();
        for r in related {
            rel_cards.push_str(&format!(
                "<div class=\"rel-item\">
                    <div class=\"rel-thumb\">
                        <div class=\"dur-badge\">{}</div>
                        <div style=\"color:#ff0000; font-size:12px; font-weight:bold; padding-top:24px; text-align:center;\">[▶]</div>
                    </div>
                    <div class=\"rel-info\">
                        <div class=\"rel-title\">{}</div>
                        <div class=\"rel-channel\">{} ✔</div>
                        <div class=\"rel-meta\">{} • {}</div>
                    </div>
                </div>",
                r.duration, r.title, r.channel, r.views, r.published
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>{} - YouTube</title>
<style>
body {{
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #0f0f0f;
    border-bottom: 1px solid #272727;
    height: 48px;
    padding: 8px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #f1f1f1; width: 32px; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #ffffff; width: 140px; }}
.logo-badge {{ font-size: 10px; color: #aaaaaa; }}
.search-container {{ display: inline-block; width: 520px; margin-left: 40px; }}
.search-pill {{ background-color: #121212; border: 1px solid #303030; border-radius: 20px; padding: 7px 18px; width: 420px; color: #f1f1f1; font-size: 14px; display: inline-block; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #3ea6ff; color: #3ea6ff; border-radius: 18px; padding: 6px 14px; font-size: 13px; font-weight: bold; margin-top: 2px; }}
.watch-container {{ padding: 18px 24px; display: block; }}
.player-column {{ display: inline-block; width: 680px; vertical-align: top; }}
.cinema-player {{
    background-color: #000000;
    border-radius: 12px;
    height: 360px;
    position: relative;
    border: 1px solid #1f1f1f;
}}
.screen-art {{
    color: #ff3333;
    font-size: 22px;
    font-weight: bold;
    padding-top: 130px;
    text-align: center;
}}
.screen-subtitle {{
    color: #ffffff;
    font-size: 15px;
    margin-top: 8px;
    text-align: center;
}}
.timeline-bar {{
    background-color: #ff0000;
    height: 4px;
    margin-top: 130px;
    width: 35%;
    display: block;
}}
.player-controls {{
    background-color: #111111;
    padding: 8px 14px;
    font-size: 13px;
    color: #f1f1f1;
    border-radius: 0 0 12px 12px;
}}
.ctrl-item {{ display: inline-block; margin-right: 14px; cursor: pointer; }}
.shield-badge {{ display: inline-block; float: right; color: #ff5500; font-weight: bold; font-size: 12px; }}
.vid-heading {{ font-size: 18px; font-weight: bold; color: #ffffff; margin-top: 14px; line-height: 1.3; }}
.channel-row {{ margin-top: 12px; padding-bottom: 12px; border-bottom: 1px solid #222222; display: block; }}
.ch-avatar-big {{
    background-color: #8b0000;
    color: #ffffff;
    border-radius: 50%;
    width: 38px;
    height: 38px;
    display: inline-block;
    text-align: center;
    line-height: 38px;
    font-weight: bold;
    vertical-align: middle;
}}
.ch-details {{ display: inline-block; vertical-align: middle; margin-left: 10px; width: 180px; }}
.ch-name {{ font-size: 14px; font-weight: bold; color: #ffffff; }}
.ch-subs {{ font-size: 11px; color: #aaaaaa; }}
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
}}
.action-pills {{ display: inline-block; float: right; vertical-align: middle; margin-top: 4px; }}
.action-pill {{
    display: inline-block;
    background-color: #272727;
    border-radius: 18px;
    padding: 7px 14px;
    font-size: 12px;
    color: #f1f1f1;
    margin-left: 6px;
}}
.desc-box {{
    background-color: #272727;
    border-radius: 12px;
    padding: 12px 16px;
    margin-top: 14px;
    font-size: 12px;
    color: #f1f1f1;
    line-height: 1.5;
}}
.related-column {{
    display: inline-block;
    width: 300px;
    vertical-align: top;
    margin-left: 20px;
}}
.rel-item {{ display: block; margin-bottom: 14px; cursor: pointer; }}
.rel-thumb {{ display: inline-block; width: 130px; height: 75px; background-color: #1a1a1a; border-radius: 8px; vertical-align: top; position: relative; }}
.dur-badge {{ background-color: #000000; color: #ffffff; border-radius: 4px; padding: 2px 4px; font-size: 10px; font-weight: bold; float: right; margin: 4px; }}
.rel-info {{ display: inline-block; width: 155px; padding-left: 10px; vertical-align: top; }}
.rel-title {{ font-size: 12px; font-weight: bold; color: #ffffff; line-height: 1.3; margin-bottom: 4px; }}
.rel-channel {{ font-size: 11px; color: #aaaaaa; margin-bottom: 2px; }}
.rel-meta {{ font-size: 10px; color: #717171; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\">[>] YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\">naki in bangla</div>
    </div>
    <div class=\"signin-btn\">[👤 Sign in]</div>
</div>

<div class=\"watch-container\">
    <div class=\"player-column\">
        <div class=\"cinema-player\">
            <div class=\"screen-art\">{}</div>
            <div class=\"screen-subtitle\">{}</div>
            <div class=\"timeline-bar\"></div>
        </div>
        <div class=\"player-controls\">
            <div class=\"ctrl-item\">{}</div>
            <div class=\"ctrl-item\">[🔊]</div>
            <div class=\"ctrl-item\">0:02 / {}</div>
            <div class=\"ctrl-item\">[CC]</div>
            <div class=\"ctrl-item\">[⚙ 1080p]</div>
            <div class=\"ctrl-item\">[⛶ Fullscreen]</div>
            <div class=\"shield-badge\">[🛡️ Brave Shields: 0 Video Ads]</div>
        </div>

        <div class=\"vid-heading\">{}</div>

        <div class=\"channel-row\">
            <div class=\"ch-avatar-big\">NAK</div>
            <div class=\"ch-details\">
                <div class=\"ch-name\">{} ✔</div>
                <div class=\"ch-subs\">778K subscribers</div>
            </div>
            <div class=\"sub-pill\">{}</div>

            <div class=\"action-pills\">
                <div class=\"action-pill\">👍 {} | 👎</div>
                <div class=\"action-pill\">↗ Share</div>
                <div class=\"action-pill\">⬇ Download</div>
                <div class=\"action-pill\">•••</div>
            </div>
        </div>

        <div class=\"desc-box\">
            <b>{} • {}</b><br>
            {}
        </div>
    </div>

    <div class=\"related-column\">
        <div style=\"font-size:14px; font-weight:bold; margin-bottom:12px;\">Up Next</div>
        {}
    </div>
</div>
</body>
</html>",
            video.title, sub_color, sub_text_color, play_badge, video.title, play_btn, video.duration,
            video.title, video.channel, sub_btn, likes, video.views, video.published, video.description, rel_cards
        )
    }

    /// Builds the rich Home Feed matching Image 5.
    #[cfg(feature = "std")]
    pub fn build_youtube_feed_html(category: &str, videos: &[YouTubeVideo]) -> String {
        let mut cards = String::new();
        for v in videos {
            let title_display = if v.title.chars().count() > 24 {
                let s: String = v.title.chars().take(22).collect();
                format!("{}...", s)
            } else {
                v.title.clone()
            };
            cards.push_str(&format!(
                "<div class=\"grid-card\">
                    <div class=\"card-thumb\">
                        <div class=\"dur-badge\">{}</div>
                        <div style=\"color:#ff0000; font-size:20px; font-weight:bold; padding-top:55px; text-align:center;\">[▶ Play]</div>
                    </div>
                    <div class=\"card-body\">
                        <div class=\"card-avatar\">[o]</div>
                        <div class=\"card-text\">
                            <div class=\"card-title\">{}</div>
                            <div class=\"card-channel\">{} ✔</div>
                            <div class=\"card-meta\">{} • {}</div>
                        </div>
                    </div>
                </div>",
                v.duration, title_display, v.channel, v.views, v.published
            ));
        }

        format!(
            "<!DOCTYPE html>
<html>
<head>
<title>YouTube</title>
<style>
body {{
    background-color: #0f0f0f;
    color: #f1f1f1;
    font-family: sans-serif;
    margin: 0;
    padding: 0;
}}
.topbar {{
    background-color: #0f0f0f;
    border-bottom: 1px solid #272727;
    height: 48px;
    padding: 8px 18px;
    display: block;
}}
.hamburger {{ display: inline-block; font-size: 20px; color: #f1f1f1; width: 32px; }}
.logo {{ display: inline-block; font-size: 19px; font-weight: bold; color: #ffffff; width: 140px; }}
.logo-badge {{ font-size: 10px; color: #aaaaaa; }}
.search-container {{ display: inline-block; width: 520px; margin-left: 40px; }}
.search-pill {{ background-color: #121212; border: 1px solid #303030; border-radius: 20px; padding: 7px 18px; width: 420px; color: #f1f1f1; font-size: 14px; display: inline-block; }}
.search-btn {{ background-color: #222222; border: 1px solid #303030; border-radius: 0 20px 20px 0; padding: 7px 18px; color: #f1f1f1; font-size: 14px; display: inline-block; margin-left: -5px; }}
.signin-btn {{ display: inline-block; float: right; border: 1px solid #3ea6ff; color: #3ea6ff; border-radius: 18px; padding: 6px 14px; font-size: 13px; font-weight: bold; margin-top: 2px; }}
.layout-row {{ display: block; }}
.sidebar {{ display: inline-block; width: 190px; background-color: #0f0f0f; border-right: 1px solid #222222; padding-top: 12px; vertical-align: top; }}
.side-item {{ padding: 10px 18px; font-size: 13px; color: #f1f1f1; border-radius: 10px; margin: 2px 10px; }}
.side-active {{ background-color: #272727; font-weight: bold; }}
.side-divider {{ border-top: 1px solid #222222; margin: 10px 14px; }}
.content-area {{ display: inline-block; width: 800px; vertical-align: top; padding: 14px 20px; }}
.chips-bar {{ margin-bottom: 18px; display: block; }}
.chip {{
    display: inline-block;
    background-color: #272727;
    border: 1px solid #383838;
    color: #f1f1f1;
    padding: 6px 14px;
    border-radius: 16px;
    font-size: 12px;
    margin-right: 6px;
    margin-bottom: 6px;
}}
.chip-active {{ background-color: #ffffff; color: #0f0f0f; font-weight: bold; }}
.grid-card {{
    display: inline-block;
    width: 250px;
    margin: 6px 8px 20px 8px;
    vertical-align: top;
}}
.card-thumb {{
    background-color: #1a1a1a;
    border-radius: 10px;
    height: 140px;
    position: relative;
    border: 1px solid #222222;
}}
.dur-badge {{ background-color: #000000; color: #ffffff; border-radius: 4px; padding: 2px 5px; font-size: 10px; font-weight: bold; float: right; margin: 6px; }}
.card-body {{ margin-top: 10px; display: block; }}
.card-avatar {{
    background-color: #333333;
    border-radius: 50%;
    width: 32px;
    height: 32px;
    display: inline-block;
    text-align: center;
    line-height: 32px;
    font-size: 11px;
    vertical-align: top;
}}
.card-text {{ display: inline-block; width: 205px; padding-left: 8px; vertical-align: top; }}
.card-title {{ font-size: 13px; font-weight: bold; color: #ffffff; line-height: 1.3; margin-bottom: 4px; }}
.card-channel {{ font-size: 11px; color: #aaaaaa; margin-bottom: 2px; }}
.card-meta {{ font-size: 10px; color: #717171; }}
</style>
</head>
<body>
<div class=\"topbar\">
    <div class=\"hamburger\">≡</div>
    <div class=\"logo\">[>] YouTube <span class=\"logo-badge\">BD</span></div>
    <div class=\"search-container\">
        <div class=\"search-pill\">Search</div>
        <div class=\"search-btn\">[Q]</div>
    </div>
    <div class=\"signin-btn\">[👤 Sign in]</div>
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
        <div class=\"side-item\">[💡 Podcasts]</div>
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
            cards
        )
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
                // 1. Watch page: /watch?v=...
                if trimmed.contains("/watch") || trimmed.contains("watch?v=") {
                    let vid = if let Some(v_part) = trimmed.split("v=").nth(1) {
                        v_part.split('&').next().unwrap_or("4NlKj7Saogc")
                    } else {
                        "4NlKj7Saogc"
                    };
                    let is_playing = !trimmed.contains("playing=false");
                    let is_sub = trimmed.contains("sub=true");
                    let likes = 4300;

                    let (video, related) = if vid == "4NlKj7Saogc" {
                        (
                            YouTubeVideo {
                                video_id: "4NlKj7Saogc".to_owned(),
                                title: "Muslim Torunder Uddesshe Kichu Kotha || Nouman Ali Khan (NAKInBangla)".to_owned(),
                                channel: "NAKInBangla".to_owned(),
                                views: "155K views".to_owned(),
                                duration: "56:31".to_owned(),
                                published: "1 year ago".to_owned(),
                                description: "Support us in patreon: https://patreon.com/spektrummedia Support us in Buzzsprout: https://buzzsprout.com... Advice and life guidance for Muslim youth by Dr. Nouman Ali Khan.".to_owned(),
                                verified: true,
                            },
                            Self::get_related_videos_for("4NlKj7Saogc")
                        )
                    } else {
                        (
                            YouTubeVideo {
                                video_id: vid.to_owned(),
                                title: "Vlad and Niki new mother explore ice cream truck (ভ্লাদ এবং নিকি)".to_owned(),
                                channel: "Niki TV".to_owned(),
                                views: "11M views".to_owned(),
                                duration: "4:47".to_owned(),
                                published: "5 years ago".to_owned(),
                                description: "Vlad and Niki explore new ice cream truck adventure for kids.".to_owned(),
                                verified: true,
                            },
                            Self::get_related_videos_for(vid)
                        )
                    };

                    let html = Self::build_youtube_watch_html(&video, &related, is_playing, is_sub, likes);
                    return (format!("{} - YouTube", video.title), html);
                }

                // 2. Search results page: search_query=...
                if trimmed.contains("search_query=") || trimmed.contains("?q=") {
                    let q = if let Some(q_part) = trimmed.split("search_query=").nth(1) {
                        q_part.split('&').next().unwrap_or("naki in bangla")
                    } else if let Some(q_part) = trimmed.split("?q=").nth(1) {
                        q_part.split('&').next().unwrap_or("naki in bangla")
                    } else {
                        "naki in bangla"
                    };
                    let decoded_q = q.replace('+', " ");
                    if let Some((yt_title, yt_html)) = Self::fetch_live_youtube(Some(&decoded_q)) {
                        return (yt_title, yt_html);
                    }
                }

                // 3. Home Feed with Filter Chips: feed=true
                if trimmed.contains("feed=true") || trimmed.contains("/feed") {
                    let videos = Self::get_default_home_feed();
                    let html = Self::build_youtube_feed_html("All", &videos);
                    return ("YouTube".to_owned(), html);
                }

                // 4. Clean Landing Page (Image 1 & 2): default or clean=true
                let q_input = if trimmed.contains("q=") {
                    trimmed.split("q=").nth(1).unwrap_or("").split('&').next().unwrap_or("")
                } else {
                    ""
                };
                let sugg_open = trimmed.contains("sugg=true") || !q_input.is_empty();
                let html = Self::build_youtube_clean_home_html(q_input, sugg_open);
                return ("YouTube".to_owned(), html);
            }
            ("YouTube".to_owned(), InternalPages::youtube_portal())
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
