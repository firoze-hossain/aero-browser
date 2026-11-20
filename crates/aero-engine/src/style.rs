//! # Aero Style Tree & Property Resolution
//!
//! Maps DOM nodes to cascading CSS rules to produce a fully resolved StyledNode tree.

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

use crate::css::{Color, CssParser, Declaration, Rule, Specificity, StyleSheet, Value};
use crate::dom::{ElementData, Node, NodeType};

/// Computed display mode for layout generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Inline,
    Block,
    None,
}

/// A node in the computed style tree.
#[derive(Debug, Clone)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: Vec<(String, Value)>,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn value(&self, name: &str) -> Option<&Value> {
        for (k, v) in &self.specified_values {
            if k == name {
                return Some(v);
            }
        }
        None
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(k)) => match k.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => {
                // HTML default display values
                if let Some(tag) = self.node.tag_name() {
                    match tag {
                        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header"
                        | "footer" | "section" | "article" | "nav" | "main" | "ul" | "ol"
                        | "li" | "form" | "table" | "pre" | "blockquote" | "hr" => Display::Block,
                        _ => Display::Inline,
                    }
                } else {
                    Display::Inline
                }
            }
        }
    }

    pub fn lookup(&self, name: &str, fallback: &str) -> Value {
        self.value(name)
            .cloned()
            .unwrap_or_else(|| Value::Keyword(fallback.to_owned()))
    }

    pub fn color(&self) -> Color {
        self.value("color")
            .and_then(|v| v.to_color())
            .unwrap_or(Color::TEXT_MAIN)
    }

    pub fn background_color(&self) -> Option<Color> {
        self.value("background-color")
            .and_then(|v| v.to_color())
            .or_else(|| self.value("background").and_then(|v| v.to_color()))
    }
}

/// Builds a styled tree from a DOM tree and a list of applied stylesheets.
pub fn build_style_tree<'a>(
    root: &'a Node,
    stylesheet: &StyleSheet,
    parent_style: Option<&StyledNode<'a>>,
) -> StyledNode<'a> {
    let mut specified_values = match &root.node_type {
        NodeType::Element(elem) => match_rules(elem, stylesheet),
        _ => Vec::new(),
    };

    // Parse inline style="..." attribute if present
    if let Some(elem) = root.element_data() {
        if let Some(inline_css) = elem.get_attr("style") {
            let inline_decls = CssParser::parse_inline_declarations(inline_css);
            for decl in inline_decls {
                // Inline styles override stylesheet rules
                if let Some(pos) = specified_values.iter().position(|(k, _)| k == &decl.name) {
                    specified_values[pos] = (decl.name, decl.value);
                } else {
                    specified_values.push((decl.name, decl.value));
                }
            }
        }
    }

    // Property inheritance from parent for text properties
    if let Some(parent) = parent_style {
        inherit_property(&mut specified_values, parent, "color");
        inherit_property(&mut specified_values, parent, "font-size");
        inherit_property(&mut specified_values, parent, "font-weight");
        inherit_property(&mut specified_values, parent, "line-height");
    }

    let mut current_styled = StyledNode {
        node: root,
        specified_values,
        children: Vec::new(),
    };

    let children = root
        .children
        .iter()
        .map(|child| build_style_tree(child, stylesheet, Some(&current_styled)))
        .collect();

    current_styled.children = children;
    current_styled
}

fn inherit_property(values: &mut Vec<(String, Value)>, parent: &StyledNode, prop: &str) {
    if !values.iter().any(|(k, _)| k == prop) {
        if let Some(parent_val) = parent.value(prop) {
            values.push((prop.to_owned(), parent_val.clone()));
        }
    }
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn match_rules<'a>(elem: &ElementData, stylesheet: &'a StyleSheet) -> Vec<(String, Value)> {
    let mut matched: Vec<MatchedRule<'a>> = Vec::new();

    for rule in &stylesheet.rules {
        for selector in &rule.selectors {
            if selector.matches(elem) {
                matched.push((selector.specificity(), rule));
                break;
            }
        }
    }

    // Sort by specificity ascending so higher specificity overwrites earlier values
    matched.sort_by(|a, b| a.0.cmp(&b.0));

    let mut values: Vec<(String, Value)> = Vec::new();
    for (_, rule) in matched {
        for decl in &rule.declarations {
            if let Some(pos) = values.iter().position(|(k, _)| k == &decl.name) {
                values[pos] = (decl.name.clone(), decl.value.clone());
            } else {
                values.push((decl.name.clone(), decl.value.clone()));
            }
        }
    }

    values
}
