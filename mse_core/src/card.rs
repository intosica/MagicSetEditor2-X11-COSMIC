use std::collections::BTreeMap;

use crate::mse_format::Node;

/// One card. Field names are game-defined (for Magic: `name`, `casting_cost`,
/// `super_type`, `sub_type`, `rule_text`, `power`, `toughness`, `image`, ...),
/// so values live in a map rather than fixed struct fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Card {
    pub fields: BTreeMap<String, String>,
    /// Per-card stylesheet override, if any.
    pub stylesheet: Option<String>,
    pub notes: String,
}

impl Card {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_node(node: &Node) -> Self {
        let mut card = Card::new();
        for c in &node.children {
            match c.key.as_str() {
                "stylesheet" => card.stylesheet = Some(c.text()),
                "notes" => card.notes = c.text(),
                "has_styling" | "styling_data" | "time_created" | "time_modified" => {}
                "" => {}
                k => {
                    card.fields.insert(k.to_owned(), c.text());
                }
            }
        }
        card
    }

    pub fn get(&self, field: &str) -> &str {
        self.fields.get(field).map(String::as_str).unwrap_or("")
    }
    pub fn set(&mut self, field: &str, value: impl Into<String>) {
        self.fields.insert(field.to_owned(), value.into());
    }
    pub fn name(&self) -> &str {
        self.get("name")
    }
    pub fn display_name(&self) -> &str {
        match self.name() {
            "" => "(unnamed card)",
            n => n,
        }
    }
}
