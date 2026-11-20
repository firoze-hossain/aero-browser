//! # Aero Layout Engine & Box Model
//!
//! Generates hierarchical layout box trees, computes CSS geometry (margins, borders,
//! padding, content area), performs block reflow, inline flow wrapping, and hit testing.

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

use crate::css::Value;
use crate::dom::NodeType;
use crate::style::{Display, StyledNode};

/// 2D rectangle in browser screen pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height }
    }

    /// Checks if a point (px, py) is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn expanded_by(&self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

/// Quad-edge sizes for margins, borders, and padding.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
    pub const fn all(val: f32) -> Self {
        EdgeSizes {
            top: val,
            right: val,
            bottom: val,
            left: val,
        }
    }
}

/// Standard CSS Box Model dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dimensions {
    /// Content area position and dimensions.
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

impl Dimensions {
    /// Returns the padding box (content + padding).
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    /// Returns the border box (content + padding + border).
    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    /// Returns the full margin box (border box + margins).
    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

/// Classification of layout boxes.
#[derive(Debug, Clone)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

/// A node in the visual layout box tree.
#[derive(Debug, Clone)]
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    pub fn new(box_type: BoxType<'a>) -> Self {
        LayoutBox {
            dimensions: Dimensions::default(),
            box_type,
            children: Vec::new(),
        }
    }

    pub fn get_styled_node(&self) -> Option<&'a StyledNode<'a>> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => Some(node),
            BoxType::AnonymousBlock => None,
        }
    }

    /// Recursively computes layout geometry based on the containing block dimensions.
    pub fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => self.layout_inline(containing_block),
            BoxType::AnonymousBlock => self.layout_block(containing_block),
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        // 1. Calculate width & margins
        self.calculate_block_width(containing_block);

        // 2. Position box relative to containing block
        self.calculate_block_position(containing_block);

        // 3. Layout child boxes
        self.layout_block_children();

        // 4. Calculate content height based on children
        self.calculate_block_height();
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        // Simple inline layout estimation
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y;
        self.dimensions.content.width = containing_block.content.width;

        let mut total_h = 18.0;
        if let Some(styled) = self.get_styled_node() {
            if let Some(tag) = styled.node.tag_name() {
                if tag == "button" {
                    self.dimensions.content.width = 120.0;
                    self.dimensions.content.height = 34.0;
                    self.dimensions.padding = EdgeSizes::all(6.0);
                    return;
                }
            }
        }
        self.dimensions.content.height = total_h;
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_styled_node();

        let auto = Value::Keyword("auto".to_owned());
        let zero = Value::Length(0.0, crate::css::Unit::Px);

        let width = style.and_then(|s| s.value("width")).unwrap_or(&auto);

        let mut margin_left = style.and_then(|s| s.value("margin-left"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero)
            .to_px();

        let mut margin_right = style.and_then(|s| s.value("margin-right"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero)
            .to_px();

        let border_left = style.and_then(|s| s.value("border-left-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero)
            .to_px();

        let border_right = style.and_then(|s| s.value("border-right-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero)
            .to_px();

        let padding_left = style.and_then(|s| s.value("padding-left"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero)
            .to_px();

        let padding_right = style.and_then(|s| s.value("padding-right"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero)
            .to_px();

        let total_non_content = margin_left + margin_right + border_left + border_right + padding_left + padding_right;

        let content_width = match width {
            Value::Length(px, crate::css::Unit::Px) => *px,
            Value::Length(pct, crate::css::Unit::Percent) => (containing_block.content.width * (pct / 100.0) - total_non_content).max(0.0),
            _ => {
                // auto width fills containing block
                (containing_block.content.width - total_non_content).max(0.0)
            }
        };

        // Center content if margin is auto
        let underflow = containing_block.content.width - (content_width + total_non_content);
        if underflow > 0.0 {
            if style.map_or(false, |s| s.lookup("margin", "0") == Value::Keyword("auto".to_owned())) {
                margin_left += underflow / 2.0;
                margin_right += underflow / 2.0;
            }
        }

        self.dimensions.content.width = content_width;
        self.dimensions.padding.left = padding_left;
        self.dimensions.padding.right = padding_right;
        self.dimensions.border.left = border_left;
        self.dimensions.border.right = border_right;
        self.dimensions.margin.left = margin_left;
        self.dimensions.margin.right = margin_right;
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_styled_node();
        let zero = Value::Length(0.0, crate::css::Unit::Px);

        let margin_top = style.and_then(|s| s.value("margin-top"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero)
            .to_px();

        let margin_bottom = style.and_then(|s| s.value("margin-bottom"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero)
            .to_px();

        let border_top = style.and_then(|s| s.value("border-top-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero)
            .to_px();

        let border_bottom = style.and_then(|s| s.value("border-bottom-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero)
            .to_px();

        let padding_top = style.and_then(|s| s.value("padding-top"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero)
            .to_px();

        let padding_bottom = style.and_then(|s| s.value("padding-bottom"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero)
            .to_px();

        self.dimensions.margin.top = margin_top;
        self.dimensions.margin.bottom = margin_bottom;
        self.dimensions.border.top = border_top;
        self.dimensions.border.bottom = border_bottom;
        self.dimensions.padding.top = padding_top;
        self.dimensions.padding.bottom = padding_bottom;

        self.dimensions.content.x = containing_block.content.x + self.dimensions.margin.left + self.dimensions.border.left + self.dimensions.padding.left;
        self.dimensions.content.y = containing_block.content.height + containing_block.content.y + self.dimensions.margin.top + self.dimensions.border.top + self.dimensions.padding.top;
    }

    fn layout_block_children(&mut self) {
        for child in &mut self.children {
            child.layout(self.dimensions);
            self.dimensions.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // If explicit CSS height property is given, override content height
        if let Some(style) = self.get_styled_node() {
            if let Some(Value::Length(h, crate::css::Unit::Px)) = style.value("height") {
                self.dimensions.content.height = *h;
            } else if self.dimensions.content.height == 0.0 {
                // Minimum non-empty height for leaf element text
                if let Some(tag) = style.node.tag_name() {
                    match tag {
                        "h1" => self.dimensions.content.height = 38.0,
                        "h2" => self.dimensions.content.height = 30.0,
                        "h3" => self.dimensions.content.height = 24.0,
                        "p" | "a" | "span" | "li" => self.dimensions.content.height = 20.0,
                        "button" => self.dimensions.content.height = 36.0,
                        "hr" => self.dimensions.content.height = 2.0,
                        _ => {}
                    }
                } else if style.node.is_text() {
                    self.dimensions.content.height = 18.0;
                }
            }
        }
    }

    /// Performs hit testing to locate which styled element exists under screen coordinate (x, y).
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&'a StyledNode<'a>> {
        if !self.dimensions.border_box().contains(x, y) {
            return None;
        }

        // Test children in reverse paint order (top-most first)
        for child in self.children.iter().rev() {
            if let Some(found) = child.hit_test(x, y) {
                return Some(found);
            }
        }

        self.get_styled_node()
    }
}

/// Builds the layout box tree from a styled node tree.
pub fn build_layout_tree<'a>(root: &'a StyledNode<'a>) -> LayoutBox<'a> {
    match root.display() {
        Display::Block => {
            let mut root_box = LayoutBox::new(BoxType::BlockNode(root));
            for child in &root.children {
                if child.display() != Display::None {
                    root_box.children.push(build_layout_tree(child));
                }
            }
            root_box
        }
        Display::Inline => {
            let mut root_box = LayoutBox::new(BoxType::InlineNode(root));
            for child in &root.children {
                if child.display() != Display::None {
                    root_box.children.push(build_layout_tree(child));
                }
            }
            root_box
        }
        Display::None => LayoutBox::new(BoxType::AnonymousBlock),
    }
}
