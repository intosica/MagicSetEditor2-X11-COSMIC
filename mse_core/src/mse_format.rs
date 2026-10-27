//! Parser for MSE's plain-text data format (used by `set`, `*.mse-game`,
//! `*.mse-style`, `*.mse-symbol`, ...).
//!
//! ```text
//! card:
//! 	name: Lightning Bolt
//! 	rule_text:
//! 		Deal 3 damage to any target.
//! ```
//! Indentation is one TAB per level. A line without `key:` form that is deeper
//! than its parent is a continuation line (multi-line value).

use crate::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    /// Text before the first `:` (empty for continuation lines).
    pub key: String,
    /// Text after the first `:`, trimmed.
    pub value: String,
    /// The entire trimmed line – needed to rebuild multi-line text that may
    /// itself contain colons.
    pub raw: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn child(&self, key: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.key == key)
    }
    pub fn children_named<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |c| c.key == key)
    }
    /// Value of this node, joining continuation lines with `\n`.
    pub fn text(&self) -> String {
        if self.children.is_empty() {
            return self.value.clone();
        }
        let mut parts: Vec<&str> = Vec::new();
        if !self.value.is_empty() {
            parts.push(&self.value);
        }
        parts.extend(self.children.iter().map(|c| c.raw.as_str()));
        parts.join("\n")
    }
    pub fn child_text(&self, key: &str) -> Option<String> {
        self.child(key).map(Node::text)
    }
}

struct Line<'a> {
    depth: usize,
    key: &'a str,
    value: &'a str,
    raw: &'a str,
}

pub fn parse(src: &str) -> Result<Vec<Node>> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut lines = Vec::new();
    for (i, l) in src.lines().enumerate() {
        let l = l.trim_end_matches('\r');
        if l.trim().is_empty() {
            continue;
        }
        let depth = l.chars().take_while(|c| *c == '\t').count();
        let raw = l.trim();
        if raw.starts_with('#') && depth == 0 {
            continue; // comment
        }
        let (key, value) = match raw.split_once(':') {
            Some((k, v)) if !k.contains(' ') || k.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '_') => {
                (k.trim(), v.trim())
            }
            _ => ("", raw),
        };
        if depth == 0 && key.is_empty() {
            return Err(Error::Parse { line: i + 1, msg: format!("expected `key: value`, got `{raw}`") });
        }
        lines.push(Line { depth, key, value, raw });
    }
    let mut pos = 0;
    Ok(build(&lines, &mut pos, 0))
}

fn build(lines: &[Line], pos: &mut usize, depth: usize) -> Vec<Node> {
    let mut out = Vec::new();
    while *pos < lines.len() {
        let l = &lines[*pos];
        if l.depth < depth {
            break;
        }
        if l.depth > depth {
            *pos += 1; // orphan deeper line
            continue;
        }
        *pos += 1;
        let mut node = Node {
            key: l.key.to_owned(),
            value: l.value.to_owned(),
            raw: l.raw.to_owned(),
            children: Vec::new(),
        };
        if *pos < lines.len() && lines[*pos].depth > depth {
            node.children = build(lines, pos, depth + 1);
        }
        out.push(node);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_and_multiline() {
        let src = "mse version: 2.0.0\ngame: magic\ncard:\n\tname: Bolt\n\trule_text:\n\t\tDeal 3 damage.\n\t\tNote: any target\n";
        let n = parse(src).unwrap();
        assert_eq!(n[1].value, "magic");
        let card = &n[2];
        assert_eq!(card.child("name").unwrap().value, "Bolt");
        assert_eq!(card.child_text("rule_text").unwrap(), "Deal 3 damage.\nNote: any target");
    }
}
