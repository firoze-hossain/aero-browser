//! # Aero CSS Parser & Cascade Engine
//!
//! Pure-Rust CSS3 tokenizer, parser, selector specificity matcher, and cascading rule engine.

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

/// 32-bit ARGB / XRGB color representation with alpha channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { a, r, g, b }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { a: 255, r, g, b }
    }

    pub const fn to_xrgb(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn to_argb(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const BRAVE_ORANGE: Color = Color::rgb(255, 85, 0);
    pub const BRAVE_LION_PURPLE: Color = Color::rgb(76, 29, 149);
    pub const CHROME_BLUE: Color = Color::rgb(26, 115, 232);
    pub const FIREFOX_ACCENT: Color = Color::rgb(255, 74, 96);
    pub const DARK_BG: Color = Color::rgb(15, 23, 42);
    pub const CARD_BG: Color = Color::rgb(30, 41, 59);
    pub const BORDER_COLOR: Color = Color::rgb(51, 65, 85);
    pub const TEXT_MAIN: Color = Color::rgb(241, 245, 249);
    pub const TEXT_MUTED: Color = Color::rgb(148, 163, 184);

    /// Parses a CSS color string (#hex, rgb(), rgba(), or standard named colors).
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        if s.starts_with('#') {
            let hex = &s[1..];
            match hex.len() {
                3 => {
                    let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                    let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                    let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                    Some(Color::rgb(r, g, b))
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(Color::rgb(r, g, b))
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(Color::rgba(r, g, b, a))
                }
                _ => None,
            }
        } else if s.starts_with("rgb(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                Some(Color::rgb(r, g, b))
            } else {
                None
            }
        } else if s.starts_with("rgba(") && s.ends_with(')') {
            let inner = &s[5..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 4 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                let a = if let Ok(alpha_float) = parts[3].parse::<f32>() {
                    (alpha_float.clamp(0.0, 1.0) * 255.0) as u8
                } else {
                    parts[3].parse::<u8>().ok()?
                };
                Some(Color::rgba(r, g, b, a))
            } else {
                None
            }
        } else {
            match s.as_str() {
                "white" => Some(Color::rgb(255, 255, 255)),
                "black" => Some(Color::rgb(0, 0, 0)),
                "red" => Some(Color::rgb(239, 68, 68)),
                "green" => Some(Color::rgb(34, 197, 94)),
                "blue" => Some(Color::rgb(59, 130, 246)),
                "yellow" => Some(Color::rgb(234, 179, 8)),
                "orange" => Some(Color::rgb(249, 115, 22)),
                "purple" => Some(Color::rgb(168, 85, 247)),
                "cyan" => Some(Color::rgb(6, 182, 212)),
                "gray" | "grey" => Some(Color::rgb(100, 116, 139)),
                "lightgray" | "lightgrey" => Some(Color::rgb(203, 213, 225)),
                "darkgray" | "darkgrey" => Some(Color::rgb(30, 41, 59)),
                "transparent" => Some(Color::TRANSPARENT),
                _ => None,
            }
        }
    }
}

/// Measurement unit for CSS Lengths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Pt,
    Em,
    Rem,
    Percent,
}

/// Evaluated CSS value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    Number(f32),
    Text(String),
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match self {
            Value::Length(val, Unit::Px) => *val,
            Value::Length(val, Unit::Pt) => *val * 1.333,
            Value::Length(val, Unit::Em) | Value::Length(val, Unit::Rem) => *val * 16.0,
            Value::Length(val, Unit::Percent) => *val, // Relative to parent
            Value::Number(val) => *val,
            _ => 0.0,
        }
    }

    pub fn to_color(&self) -> Option<Color> {
        match self {
            Value::ColorValue(c) => Some(*c),
            Value::Keyword(k) => Color::parse(k),
            _ => None,
        }
    }
}

/// Selector specificity triple: (a: IDs, b: Classes & Attributes, c: Tag names).
pub type Specificity = (usize, usize, usize);

/// Simple CSS selector representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

impl SimpleSelector {
    pub fn specificity(&self) -> Specificity {
        let a = if self.id.is_some() { 1 } else { 0 };
        let b = self.classes.len();
        let c = if self.tag_name.is_some() && self.tag_name.as_deref() != Some("*") { 1 } else { 0 };
        (a, b, c)
    }

    pub fn matches(&self, elem: &crate::dom::ElementData) -> bool {
        if let Some(ref tag) = self.tag_name {
            if tag != "*" && elem.tag_name != *tag {
                return false;
            }
        }
        if let Some(ref id) = self.id {
            if elem.id() != Some(id.as_str()) {
                return false;
            }
        }
        for class in &self.classes {
            if !elem.has_class(class) {
                return false;
            }
        }
        true
    }
}

/// A CSS declaration `property: value;`.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

/// A CSS rule combining a list of selectors and a declaration block.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<SimpleSelector>,
    pub declarations: Vec<Declaration>,
}

/// A parsed stylesheet holding multiple styling rules.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        StyleSheet { rules: Vec::new() }
    }

    /// Appends a new rule to the stylesheet.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
}

/// Fast parser for CSS stylesheets and inline `style="..."` attributes.
pub struct CssParser {
    chars: Vec<char>,
    pos: usize,
}

impl CssParser {
    pub fn new(css: &str) -> Self {
        CssParser {
            chars: css.chars().collect(),
            pos: 0,
        }
    }

    pub fn parse(css: &str) -> StyleSheet {
        let mut parser = CssParser::new(css);
        parser.parse_stylesheet()
    }

    /// Parses an inline style string (e.g. `color: red; padding: 10px;`) into declarations.
    pub fn parse_inline_declarations(style_attr: &str) -> Vec<Declaration> {
        let mut parser = CssParser::new(style_attr);
        parser.parse_declarations()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else if c == '/' && self.chars.get(self.pos + 1) == Some(&'*') {
                // Skip CSS comment `/* ... */`
                self.pos += 2;
                while self.pos < self.chars.len() {
                    if self.chars[self.pos] == '*' && self.chars.get(self.pos + 1) == Some(&'/') {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn parse_stylesheet(&mut self) -> StyleSheet {
        let mut sheet = StyleSheet::new();
        while self.pos < self.chars.len() {
            self.consume_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }
            if let Some(rule) = self.parse_rule() {
                sheet.add_rule(rule);
            }
        }
        sheet
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selectors();
        if selectors.is_empty() {
            return None;
        }
        self.consume_whitespace();
        if self.peek() == Some('{') {
            self.next();
        }
        let declarations = self.parse_declarations();
        self.consume_whitespace();
        if self.peek() == Some('}') {
            self.next();
        }
        Some(Rule {
            selectors,
            declarations,
        })
    }

    fn parse_selectors(&mut self) -> Vec<SimpleSelector> {
        let mut selectors = Vec::new();
        loop {
            self.consume_whitespace();
            if let Some(sel) = self.parse_simple_selector() {
                selectors.push(sel);
            }
            self.consume_whitespace();
            if self.peek() == Some(',') {
                self.next();
            } else {
                break;
            }
        }
        selectors
    }

    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut selector = SimpleSelector::default();
        let mut has_part = false;

        while self.pos < self.chars.len() {
            self.consume_whitespace();
            match self.peek() {
                Some('#') => {
                    self.next();
                    selector.id = Some(self.parse_identifier());
                    has_part = true;
                }
                Some('.') => {
                    self.next();
                    selector.classes.push(self.parse_identifier());
                    has_part = true;
                }
                Some('*') => {
                    self.next();
                    selector.tag_name = Some("*".to_owned());
                    has_part = true;
                }
                Some(c) if c.is_alphabetic() => {
                    selector.tag_name = Some(self.parse_identifier().to_lowercase());
                    has_part = true;
                }
                _ => break,
            }
        }

        if has_part {
            Some(selector)
        } else {
            None
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.peek() == Some('}') || self.peek().is_none() {
                break;
            }

            let name = self.parse_identifier();
            if name.is_empty() {
                self.next();
                continue;
            }

            self.consume_whitespace();
            if self.peek() == Some(':') {
                self.next();
            }

            self.consume_whitespace();
            let val = self.parse_value();
            declarations.push(Declaration {
                name: name.to_lowercase(),
                value: val,
            });

            self.consume_whitespace();
            if self.peek() == Some(';') {
                self.next();
            }
        }
        declarations
    }

    fn parse_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                ident.push(c);
                self.next();
            } else {
                break;
            }
        }
        ident
    }

    fn parse_value(&mut self) -> Value {
        self.consume_whitespace();
        let mut raw = String::new();

        while let Some(c) = self.peek() {
            if c == ';' || c == '}' || c == '!' {
                break;
            }
            raw.push(c);
            self.next();
        }

        let s = raw.trim();

        // Check for length/number
        if let Some(val) = Self::try_parse_length(s) {
            return val;
        }

        // Check for color
        if let Some(col) = Color::parse(s) {
            return Value::ColorValue(col);
        }

        Value::Keyword(s.to_owned())
    }

    fn try_parse_length(s: &str) -> Option<Value> {
        if s.ends_with("px") {
            let num = s[..s.len() - 2].trim().parse::<f32>().ok()?;
            Some(Value::Length(num, Unit::Px))
        } else if s.ends_with("pt") {
            let num = s[..s.len() - 2].trim().parse::<f32>().ok()?;
            Some(Value::Length(num, Unit::Pt))
        } else if s.ends_with("rem") {
            let num = s[..s.len() - 3].trim().parse::<f32>().ok()?;
            Some(Value::Length(num, Unit::Rem))
        } else if s.ends_with("em") {
            let num = s[..s.len() - 2].trim().parse::<f32>().ok()?;
            Some(Value::Length(num, Unit::Em))
        } else if s.ends_with('%') {
            let num = s[..s.len() - 1].trim().parse::<f32>().ok()?;
            Some(Value::Length(num, Unit::Percent))
        } else if let Ok(num) = s.parse::<f32>() {
            Some(Value::Number(num))
        } else {
            None
        }
    }
}

/// Default built-in user agent stylesheet providing clean, modern browser typography.
pub fn default_user_agent_stylesheet() -> StyleSheet {
    let css = "
html, body {
    margin: 0;
    padding: 0;
    background-color: #0f172a;
    color: #f1f5f9;
    font-size: 16px;
    line-height: 1.5;
}
h1 {
    font-size: 32px;
    font-weight: bold;
    margin: 16px 0 8px 0;
    color: #ffffff;
}
h2 {
    font-size: 24px;
    font-weight: bold;
    margin: 14px 0 6px 0;
    color: #f8fafc;
}
h3 {
    font-size: 20px;
    font-weight: bold;
    margin: 12px 0 4px 0;
    color: #e2e8f0;
}
h4, h5, h6 {
    font-size: 16px;
    font-weight: bold;
    margin: 8px 0 4px 0;
}
p {
    margin: 8px 0;
    color: #cbd5e1;
}
a {
    color: #38bdf8;
    text-decoration: underline;
}
button {
    background-color: #ff5500;
    color: #ffffff;
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
}
input {
    background-color: #1e293b;
    color: #f1f5f9;
    padding: 8px 12px;
    border: 1px solid #475569;
    border-radius: 6px;
}
code, pre {
    background-color: #1e293b;
    color: #38bdf8;
    padding: 4px 8px;
    border-radius: 4px;
    font-family: monospace;
}
hr {
    border: none;
    border-top: 1px solid #334155;
    margin: 16px 0;
}
ul, ol {
    margin: 8px 0;
    padding-left: 24px;
}
li {
    margin: 4px 0;
}
.card {
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 8px;
    padding: 16px;
    margin: 12px 0;
}
.btn-primary {
    background-color: #ff5500;
    color: #ffffff;
}
.btn-secondary {
    background-color: #334155;
    color: #f1f5f9;
}
.text-muted {
    color: #94a3b8;
}
";
    CssParser::parse(css)
}
