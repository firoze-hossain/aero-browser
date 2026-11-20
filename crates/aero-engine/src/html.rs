//! # Aero HTML Parser & Tree Builder
//!
//! A fast, zero-dependency pure Rust HTML5 tokenizer and DOM tree builder.
//! Safely handles arbitrary HTML, void elements, character entities, inline styles,
//! and unclosed tags without crashing.

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

use crate::dom::{ElementData, Node, NodeType};

/// Decodes common HTML entities into their Unicode characters.
pub fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            let mut found_semi = false;
            while let Some(&next_c) = chars.peek() {
                if next_c == ';' {
                    chars.next();
                    found_semi = true;
                    break;
                } else if next_c.is_alphanumeric() || next_c == '#' {
                    entity.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            match entity.as_str() {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                "nbsp" => out.push(' '),
                "copy" => out.push('©'),
                "reg" => out.push('®'),
                "trade" => out.push('™'),
                "bull" => out.push('•'),
                "mdash" => out.push('—'),
                "ndash" => out.push('–'),
                "hellip" => out.push('…'),
                s if s.starts_with("#x") || s.starts_with("#X") => {
                    if let Ok(val) = u32::from_str_radix(&s[2..], 16) {
                        if let Some(ch) = core::char::from_u32(val) {
                            out.push(ch);
                        }
                    }
                }
                s if s.starts_with('#') => {
                    if let Ok(val) = s[1..].parse::<u32>() {
                        if let Some(ch) = core::char::from_u32(val) {
                            out.push(ch);
                        }
                    }
                }
                _ => {
                    out.push('&');
                    out.push_str(&entity);
                    if found_semi {
                        out.push(';');
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Tokenizer and DOM tree builder parser state.
pub struct HtmlParser {
    chars: Vec<char>,
    pos: usize,
}

impl HtmlParser {
    /// Constructs a parser instance from raw HTML text.
    pub fn new(html: &str) -> Self {
        HtmlParser {
            chars: html.chars().collect(),
            pos: 0,
        }
    }

    /// Parses the HTML string into a root Document DOM node.
    pub fn parse(html: &str) -> Node {
        let mut parser = HtmlParser::new(html);
        let nodes = parser.parse_nodes();
        Node::new_document(nodes)
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

    fn starts_with(&self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        if self.pos + s_chars.len() > self.chars.len() {
            return false;
        }
        for (i, &sc) in s_chars.iter().enumerate() {
            if self.chars[self.pos + i] != sc {
                return false;
            }
        }
        true
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        while self.pos < self.chars.len() {
            self.consume_whitespace();
            if self.pos >= self.chars.len() {
                break;
            }

            if self.starts_with("<!--") {
                if let Some(comment) = self.parse_comment() {
                    nodes.push(comment);
                }
            } else if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
                self.consume_until_char('>');
                self.next(); // skip '>'
            } else if self.starts_with("</") {
                // End tag encountered without matching start tag at this level; return to caller
                break;
            } else if self.peek() == Some('<') {
                if let Some(element) = self.parse_element() {
                    nodes.push(element);
                }
            } else {
                let text = self.parse_text();
                if !text.trim().is_empty() {
                    nodes.push(Node::new_text(&decode_entities(&text)));
                }
            }
        }
        nodes
    }

    fn parse_comment(&mut self) -> Option<Node> {
        self.pos += 4; // Skip "<!--"
        let mut comment_text = String::new();
        while self.pos < self.chars.len() {
            if self.starts_with("-->") {
                self.pos += 3;
                return Some(Node::new_comment(comment_text.trim()));
            }
            if let Some(c) = self.next() {
                comment_text.push(c);
            }
        }
        Some(Node::new_comment(comment_text.trim()))
    }

    fn parse_text(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c == '<' {
                break;
            }
            text.push(c);
            self.next();
        }
        text
    }

    fn parse_element(&mut self) -> Option<Node> {
        self.next(); // Skip '<'
        let tag_name = self.parse_tag_name();
        if tag_name.is_empty() {
            return None;
        }

        let attributes = self.parse_attributes();

        let mut is_self_closing = false;
        self.consume_whitespace();
        if self.peek() == Some('/') {
            self.next();
            is_self_closing = true;
        }
        if self.peek() == Some('>') {
            self.next();
        }

        // HTML void elements that never have closing tags
        let is_void = match tag_name.to_lowercase().as_str() {
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta"
            | "param" | "source" | "track" | "wbr" => true,
            _ => false,
        };

        let mut children = Vec::new();
        if !is_self_closing && !is_void {
            if tag_name.to_lowercase() == "script" || tag_name.to_lowercase() == "style" {
                // Consume raw text until closing tag
                let raw_content = self.consume_raw_until_closing_tag(&tag_name);
                children.push(Node::new_text(&raw_content));
            } else {
                // Parse nested child nodes
                children = self.parse_nodes();
                self.consume_closing_tag(&tag_name);
            }
        }

        Some(Node {
            node_type: NodeType::Element(ElementData::new(tag_name, attributes)),
            children,
        })
    }

    fn parse_tag_name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' {
                name.push(c);
                self.next();
            } else {
                break;
            }
        }
        name
    }

    fn parse_attributes(&mut self) -> Vec<(String, String)> {
        let mut attributes = Vec::new();

        loop {
            self.consume_whitespace();
            if self.peek() == Some('>') || self.peek() == Some('/') || self.peek().is_none() {
                break;
            }

            let attr_name = self.parse_tag_name();
            if attr_name.is_empty() {
                self.next();
                continue;
            }

            self.consume_whitespace();
            let attr_val = if self.peek() == Some('=') {
                self.next(); // Skip '='
                self.consume_whitespace();
                self.parse_attribute_value()
            } else {
                // Boolean attribute (e.g. `disabled`, `checked`)
                attr_name.clone()
            };

            attributes.push((attr_name, decode_entities(&attr_val)));
        }

        attributes
    }

    fn parse_attribute_value(&mut self) -> String {
        let quote = self.peek();
        if quote == Some('"') || quote == Some('\'') {
            let quote_char = quote.unwrap();
            self.next(); // Skip open quote
            let mut val = String::new();
            while let Some(c) = self.peek() {
                if c == quote_char {
                    self.next(); // Skip close quote
                    break;
                }
                val.push(c);
                self.next();
            }
            val
        } else {
            // Unquoted attribute value
            let mut val = String::new();
            while let Some(c) = self.peek() {
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                val.push(c);
                self.next();
            }
            val
        }
    }

    fn consume_until_char(&mut self, target: char) {
        while let Some(c) = self.peek() {
            if c == target {
                break;
            }
            self.next();
        }
    }

    fn consume_closing_tag(&mut self, expected_tag: &str) {
        self.consume_whitespace();
        if self.starts_with("</") {
            self.pos += 2;
            let tag = self.parse_tag_name();
            self.consume_until_char('>');
            if self.peek() == Some('>') {
                self.next();
            }
        }
    }

    fn consume_raw_until_closing_tag(&mut self, tag: &str) -> String {
        let close_needle = format!("</{}", tag.to_lowercase());
        let mut content = String::new();

        while self.pos < self.chars.len() {
            let remaining: String = self.chars[self.pos..].iter().take(close_needle.len()).collect();
            if remaining.to_lowercase() == close_needle {
                self.pos += close_needle.len();
                self.consume_until_char('>');
                if self.peek() == Some('>') {
                    self.next();
                }
                break;
            }
            if let Some(c) = self.next() {
                content.push(c);
            }
        }
        content
    }
}
