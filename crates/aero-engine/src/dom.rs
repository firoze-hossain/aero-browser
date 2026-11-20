//! # Aero DOM (Document Object Model)
//!
//! Pure-Rust, lightweight representation of HTML nodes, elements, attributes,
//! and hierarchy trees with fast traversal and query selectors.

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

/// Type of DOM node in the document tree.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    /// Root document node containing child elements (html, head, body).
    Document,
    /// HTML element node with tag name and attribute key-value pairs.
    Element(ElementData),
    /// Text node containing plain character data.
    Text(String),
    /// HTML comment node `<!-- comment -->`.
    Comment(String),
}

/// Data payload for an HTML element node.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    /// Lowercase tag name, e.g. "div", "h1", "p", "a".
    pub tag_name: String,
    /// Attribute list (e.g. `[("class", "btn active"), ("id", "submit")]`).
    pub attributes: Vec<(String, String)>,
}

impl ElementData {
    /// Creates a new element data container with the given tag name and attributes.
    pub fn new(tag_name: String, attributes: Vec<(String, String)>) -> Self {
        ElementData {
            tag_name: tag_name.to_lowercase(),
            attributes,
        }
    }

    /// Looks up an attribute value by name (case-insensitive key).
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        for (k, v) in &self.attributes {
            if k.to_lowercase() == name_lower {
                return Some(v.as_str());
            }
        }
        None
    }

    /// Returns the element's `id` attribute if set.
    pub fn id(&self) -> Option<&str> {
        self.get_attr("id")
    }

    /// Returns all CSS class names listed in the `class` attribute.
    pub fn classes(&self) -> Vec<&str> {
        match self.get_attr("class") {
            Some(classes) => classes.split_whitespace().collect(),
            None => Vec::new(),
        }
    }

    /// Checks if this element possesses the specified CSS class.
    pub fn has_class(&self, class_name: &str) -> bool {
        self.classes().iter().any(|&c| c == class_name)
    }

    /// Returns the hyperlink destination `href` if this element has one.
    pub fn href(&self) -> Option<&str> {
        self.get_attr("href")
    }

    /// Returns the image source `src` if this element has one.
    pub fn src(&self) -> Option<&str> {
        self.get_attr("src")
    }
}

/// A node in the Aero DOM tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Node discriminator and specific payload.
    pub node_type: NodeType,
    /// Ordered child nodes nested under this node.
    pub children: Vec<Node>,
}

impl Node {
    /// Constructs a new element node with specified tag, attributes, and children.
    pub fn new_element(tag: &str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Self {
        let attr_vec = attrs
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Node {
            node_type: NodeType::Element(ElementData::new(tag.to_owned(), attr_vec)),
            children,
        }
    }

    /// Constructs a text node.
    pub fn new_text(text: &str) -> Self {
        Node {
            node_type: NodeType::Text(text.to_owned()),
            children: Vec::new(),
        }
    }

    /// Constructs a document root node.
    pub fn new_document(children: Vec<Node>) -> Self {
        Node {
            node_type: NodeType::Document,
            children,
        }
    }

    /// Constructs a comment node.
    pub fn new_comment(comment: &str) -> Self {
        Node {
            node_type: NodeType::Comment(comment.to_owned()),
            children: Vec::new(),
        }
    }

    /// Convenience check if node is an element.
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element(_))
    }

    /// Convenience check if node is text.
    pub fn is_text(&self) -> bool {
        matches!(self.node_type, NodeType::Text(_))
    }

    /// Returns element data reference if this node is an element.
    pub fn element_data(&self) -> Option<&ElementData> {
        match &self.node_type {
            NodeType::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns mutable element data reference if this node is an element.
    pub fn element_data_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.node_type {
            NodeType::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns tag name if this is an element node.
    pub fn tag_name(&self) -> Option<&str> {
        self.element_data().map(|e| e.tag_name.as_str())
    }

    /// Appends a child node to this node's children list.
    pub fn append_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Recursive search for the first element matching `id`.
    pub fn get_element_by_id(&self, id: &str) -> Option<&Node> {
        if let Some(elem) = self.element_data() {
            if elem.id() == Some(id) {
                return Some(self);
            }
        }
        for child in &self.children {
            if let Some(found) = child.get_element_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    /// Recursive collection of all elements matching the specified tag name.
    pub fn get_elements_by_tag_name<'a>(&'a self, tag: &str) -> Vec<&'a Node> {
        let mut results = Vec::new();
        let target_tag = tag.to_lowercase();
        self.collect_by_tag(&target_tag, &mut results);
        results
    }

    fn collect_by_tag<'a>(&'a self, tag: &str, results: &mut Vec<&'a Node>) {
        if let Some(elem) = self.element_data() {
            if tag == "*" || elem.tag_name == tag {
                results.push(self);
            }
        }
        for child in &self.children {
            child.collect_by_tag(tag, results);
        }
    }

    /// Recursive collection of all elements containing the given CSS class.
    pub fn get_elements_by_class_name<'a>(&'a self, class: &str) -> Vec<&'a Node> {
        let mut results = Vec::new();
        self.collect_by_class(class, &mut results);
        results
    }

    fn collect_by_class<'a>(&'a self, class: &str, results: &mut Vec<&'a Node>) {
        if let Some(elem) = self.element_data() {
            if elem.has_class(class) {
                results.push(self);
            }
        }
        for child in &self.children {
            child.collect_by_class(class, results);
        }
    }

    /// Extracts all concatenated plain text contained within this node and its descendants.
    pub fn text_content(&self) -> String {
        let mut text = String::new();
        self.collect_text(&mut text);
        text
    }

    fn collect_text(&self, out: &mut String) {
        match &self.node_type {
            NodeType::Text(s) => out.push_str(s),
            NodeType::Element(_) | NodeType::Document => {
                for child in &self.children {
                    child.collect_text(out);
                }
            }
            NodeType::Comment(_) => {}
        }
    }
}
