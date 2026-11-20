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

    /// Recursively translates this box and all its children along the X axis.
    pub fn translate_x(&mut self, dx: f32) {
        self.dimensions.content.x += dx;
        for child in &mut self.children {
            child.translate_x(dx);
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
        // 1. Calculate width & horizontal margins/borders/padding
        self.calculate_block_width(containing_block);

        // 2. Position box relative to containing block
        self.calculate_block_position(containing_block);

        // 3. Layout child boxes
        self.layout_block_children();

        // 4. Calculate content height based on children or text
        self.calculate_block_height();
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        let style = self.get_styled_node();
        let zero = Value::Length(0.0, crate::css::Unit::Px);

        let pad_top = style.and_then(|s| s.value("padding-top"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero).to_px();
        let pad_bottom = style.and_then(|s| s.value("padding-bottom"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero).to_px();
        let pad_left = style.and_then(|s| s.value("padding-left"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero).to_px();
        let pad_right = style.and_then(|s| s.value("padding-right"))
            .or_else(|| style.and_then(|s| s.value("padding")))
            .unwrap_or(&zero).to_px();
        self.dimensions.padding = EdgeSizes {
            top: pad_top,
            bottom: pad_bottom,
            left: pad_left,
            right: pad_right,
        };

        let border_top = style.and_then(|s| s.value("border-top-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero).to_px();
        let border_bottom = style.and_then(|s| s.value("border-bottom-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero).to_px();
        let border_left = style.and_then(|s| s.value("border-left-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero).to_px();
        let border_right = style.and_then(|s| s.value("border-right-width"))
            .or_else(|| style.and_then(|s| s.value("border-width")))
            .unwrap_or(&zero).to_px();
        self.dimensions.border = EdgeSizes {
            top: border_top,
            bottom: border_bottom,
            left: border_left,
            right: border_right,
        };

        let margin_top = style.and_then(|s| s.value("margin-top"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero).to_px();
        let margin_bottom = style.and_then(|s| s.value("margin-bottom"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .unwrap_or(&zero).to_px();
        let margin_left = style.and_then(|s| s.value("margin-left"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .map(|v| match v {
                Value::Length(pct, crate::css::Unit::Percent) => {
                    containing_block.content.width * (*pct / 100.0)
                }
                _ => v.to_px(),
            })
            .unwrap_or(0.0);
        let margin_right = style.and_then(|s| s.value("margin-right"))
            .or_else(|| style.and_then(|s| s.value("margin")))
            .map(|v| match v {
                Value::Length(pct, crate::css::Unit::Percent) => {
                    containing_block.content.width * (*pct / 100.0)
                }
                _ => v.to_px(),
            })
            .unwrap_or(0.0);
        self.dimensions.margin = EdgeSizes {
            top: margin_top,
            bottom: margin_bottom,
            left: margin_left,
            right: margin_right,
        };

        self.dimensions.content.x = containing_block.content.x + self.dimensions.margin.left + self.dimensions.border.left + self.dimensions.padding.left;
        self.dimensions.content.y = containing_block.content.y + self.dimensions.margin.top + self.dimensions.border.top + self.dimensions.padding.top;

        if let Some(s) = style {
            if let crate::dom::NodeType::Text(ref text) = s.node.node_type {
                let font_size = s.value("font-size").map(|v| v.to_px()).unwrap_or(15.0);
                let (tw, th) = crate::font::measure_text(text.trim(), font_size);
                self.dimensions.content.width = tw;
                self.dimensions.content.height = th;
                let is_center = s.lookup("text-align", "left") == crate::css::Value::Keyword("center".to_owned());
                if is_center && containing_block.content.width > tw {
                    let offset_x = (containing_block.content.width - tw) / 2.0;
                    self.dimensions.content.x = containing_block.content.x + offset_x;
                }
                return;
            }
        }

        let is_border_box = style.map_or(true, |s| {
            s.lookup("box-sizing", "border-box") == Value::Keyword("border-box".to_owned())
        });
        let total_non_content = if is_border_box {
            self.dimensions.padding.left + self.dimensions.padding.right
                + self.dimensions.border.left + self.dimensions.border.right
        } else {
            0.0
        };

        let mut explicit_width = 0.0f32;
        if let Some(s) = style {
            if let Some(w) = s.value("width") {
                explicit_width = match w {
                    Value::Length(pct, crate::css::Unit::Percent) => {
                        ((containing_block.content.width * (*pct / 100.0)) - total_non_content).max(0.0)
                    }
                    _ => (w.to_px() - total_non_content).max(0.0),
                };
            }
            if let Some(h) = s.value("height") {
                let eh = (h.to_px() - if is_border_box {
                    self.dimensions.padding.top + self.dimensions.padding.bottom
                        + self.dimensions.border.top + self.dimensions.border.bottom
                } else { 0.0 }).max(0.0);
                if eh > 0.0 {
                    self.dimensions.content.height = eh;
                }
            }
        }
        if explicit_width > 0.0 {
            self.dimensions.content.width = explicit_width;
        }

        let has_block_children = self.children.iter().any(|c| matches!(c.box_type, BoxType::BlockNode(_)));
        if has_block_children {
            // Stack block children vertically inside this inline-block container
            let mut cur_y = self.dimensions.content.y;
            let mut max_w = explicit_width;
            for child in &mut self.children {
                let mut child_cb = self.dimensions;
                child_cb.content.x = self.dimensions.content.x;
                child_cb.content.y = cur_y;
                child.layout(child_cb);

                let mb = child.dimensions.margin_box();
                cur_y += mb.height + 6.0;
                max_w = max_w.max(mb.width);
            }
            self.dimensions.content.width = max_w.max(self.dimensions.content.width);
            self.dimensions.content.height = (cur_y - self.dimensions.content.y).max(30.0);
            return;
        }

        // Layout any nested children
        let mut cur_x = self.dimensions.content.x;
        let mut cur_y = self.dimensions.content.y;
        let mut max_row_h: f32 = 20.0;
        let mut max_w: f32 = 0.0;

        let container_right = if explicit_width > 0.0 {
            self.dimensions.content.x + explicit_width
        } else {
            containing_block.content.x + containing_block.content.width
        };

        for child in &mut self.children {
            let mut child_cb = self.dimensions;
            child_cb.content.x = cur_x;
            child_cb.content.y = cur_y;
            child.layout(child_cb);

            let mb = child.dimensions.margin_box();
            max_row_h = max_row_h.max(mb.height);

            if cur_x + mb.width > container_right && cur_x > self.dimensions.content.x {
                cur_x = self.dimensions.content.x;
                cur_y += max_row_h;
                max_row_h = mb.height;
                child.dimensions.content.x = cur_x;
                child.dimensions.content.y = cur_y;
            }

            cur_x += mb.width;
            max_w = max_w.max(cur_x - self.dimensions.content.x);
        }

        if explicit_width > 0.0 {
            self.dimensions.content.width = explicit_width;
        } else {
            self.dimensions.content.width = max_w.max(20.0);
        }
        if self.dimensions.content.height == 0.0 {
            self.dimensions.content.height = (cur_y + max_row_h - self.dimensions.content.y).max(20.0);
        }
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

        let mut content_width = match width {
            Value::Length(px, crate::css::Unit::Px) => *px,
            Value::Length(pct, crate::css::Unit::Percent) => (containing_block.content.width * (pct / 100.0) - total_non_content).max(0.0),
            _ => (containing_block.content.width - total_non_content).max(0.0),
        };

        // Support max-width (e.g. max-width: 840px on .dashboard)
        if let Some(max_val) = style.and_then(|s| s.value("max-width")) {
            let max_w = max_val.to_px();
            if max_w > 0.0 && content_width > max_w {
                content_width = max_w;
            }
        }

        // Center content if margin-left and margin-right are auto or margin is 0 auto
        let underflow = containing_block.content.width - (content_width + total_non_content);
        if underflow > 0.0 {
            let has_auto_margin = style.map_or(false, |s| {
                matches!(s.lookup("margin", "0"), Value::Keyword(ref k) if k == "auto" || k.contains("auto"))
                    || s.lookup("margin-left", "") == Value::Keyword("auto".to_owned())
            });
            if has_auto_margin {
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
        self.dimensions.content.y = containing_block.content.y + self.dimensions.margin.top + self.dimensions.border.top + self.dimensions.padding.top;
    }

    fn layout_block_children(&mut self) {
        let mut cur_y = self.dimensions.content.y;
        let mut cur_x = self.dimensions.content.x;
        let mut right_cur_x = self.dimensions.content.x + self.dimensions.content.width;
        let mut row_h: f32 = 0.0;

        for child in &mut self.children {
            let is_float_right = child.get_styled_node().map_or(false, |s| {
                s.lookup("float", "none") == Value::Keyword("right".to_owned())
            });

            if is_float_right {
                let mut child_cb = self.dimensions;
                child_cb.content.x = self.dimensions.content.x;
                child_cb.content.y = cur_y;
                child.layout(child_cb);

                let mb = child.dimensions.margin_box();
                let right_x = right_cur_x - mb.width - child.dimensions.margin.right;
                let delta_x = right_x - (child.dimensions.content.x - child.dimensions.padding.left - child.dimensions.border.left - child.dimensions.margin.left);
                if delta_x > 0.0 {
                    child.translate_x(delta_x);
                }
                right_cur_x = right_x - child.dimensions.margin.left;
                row_h = row_h.max(mb.height);
                continue;
            }

            let is_inline_flow = matches!(child.box_type, BoxType::InlineNode(_));

            if is_inline_flow {
                // Inline / inline-block child: layout along horizontal row
                let mut child_cb = self.dimensions;
                child_cb.content.x = cur_x;
                child_cb.content.y = cur_y;
                child.layout(child_cb);

                let mb = child.dimensions.margin_box();
                row_h = row_h.max(mb.height);

                if cur_x + mb.width > self.dimensions.content.x + self.dimensions.content.width && cur_x > self.dimensions.content.x {
                    // Wrap to next row
                    cur_x = self.dimensions.content.x;
                    cur_y += row_h;
                    right_cur_x = self.dimensions.content.x + self.dimensions.content.width;
                    row_h = mb.height;
                    let mut child_cb = self.dimensions;
                    child_cb.content.x = cur_x;
                    child_cb.content.y = cur_y;
                    child.layout(child_cb);
                }
                cur_x += mb.width;
            } else {
                // Block child: end current inline row if any, stack vertically
                if row_h > 0.0 {
                    cur_y += row_h + 8.0;
                    right_cur_x = self.dimensions.content.x + self.dimensions.content.width;
                    row_h = 0.0;
                    cur_x = self.dimensions.content.x;
                }
                let mut child_cb = self.dimensions;
                child_cb.content.y = cur_y;
                child.layout(child_cb);

                cur_y += child.dimensions.margin_box().height;
            }
        }

        if row_h > 0.0 {
            cur_y += row_h;
        }

        self.dimensions.content.height = (cur_y - self.dimensions.content.y).max(0.0);
    }

    fn calculate_block_height(&mut self) {
        if let Some(style) = self.get_styled_node() {
            if let Some(Value::Length(h, crate::css::Unit::Px)) = style.value("height") {
                self.dimensions.content.height = *h;
            } else if self.dimensions.content.height == 0.0 {
                if let Some(tag) = style.node.tag_name() {
                    match tag {
                        "h1" => self.dimensions.content.height = 42.0,
                        "h2" => self.dimensions.content.height = 32.0,
                        "h3" => self.dimensions.content.height = 26.0,
                        "p" | "a" | "span" | "li" => self.dimensions.content.height = 22.0,
                        "button" => self.dimensions.content.height = 36.0,
                        "hr" => self.dimensions.content.height = 2.0,
                        _ => {}
                    }
                } else if style.node.is_text() {
                    self.dimensions.content.height = 20.0;
                }
            }
        }
    }

    /// Performs hit testing to locate which styled element exists under screen coordinate (x, y).
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&'a StyledNode<'a>> {
        if !self.dimensions.border_box().contains(x, y) {
            return None;
        }

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
