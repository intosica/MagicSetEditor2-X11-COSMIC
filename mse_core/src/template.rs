//! Game (`*.mse-game`) and stylesheet (`*.mse-style`) definitions.
//! Only the structural skeleton is parsed so far; MSE's script language
//! (`script:` blocks) is a later milestone.

use crate::mse_format::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub kind: String, // text, choice, image, symbol, ...
}

#[derive(Debug, Clone, Default)]
pub struct Game {
    pub name: String,
    pub card_fields: Vec<FieldDef>,
}

/// Position of one field on the card (pixels at `card_dpi`).
#[derive(Debug, Clone, PartialEq)]
pub struct FieldStyle {
    pub field: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct StyleSheet {
    pub name: String,
    pub card_width: u32,
    pub card_height: u32,
    pub card_dpi: f32,
    pub styles: Vec<FieldStyle>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub game: Game,
    pub stylesheet: StyleSheet,
}

impl Game {
    pub fn from_nodes(nodes: &[Node]) -> Self {
        let mut g = Game::default();
        for n in nodes {
            match n.key.as_str() {
                "full_name" | "name" if g.name.is_empty() => g.name = n.text(),
                "card_field" => g.card_fields.push(FieldDef {
                    name: n.child_text("name").unwrap_or_default(),
                    kind: n.child_text("type").unwrap_or_default(),
                }),
                _ => {}
            }
        }
        g
    }
}

impl StyleSheet {
    pub fn from_nodes(name: &str, nodes: &[Node]) -> Self {
        let mut s = Self::default_for(name);
        for n in nodes {
            match n.key.as_str() {
                "card_width" => s.card_width = n.value.parse().unwrap_or(s.card_width),
                "card_height" => s.card_height = n.value.parse().unwrap_or(s.card_height),
                "card_dpi" => s.card_dpi = n.value.parse().unwrap_or(s.card_dpi),
                _ => {}
            }
        }
        // TODO: `<field> style:` blocks -> FieldStyle (+ script expressions)
        s
    }

    fn default_for(name: &str) -> Self {
        // 63x88 mm at ~150 dpi
        Self { name: name.to_owned(), card_width: 375, card_height: 523, card_dpi: 150.0, styles: Vec::new() }
    }
}

impl Template {
    /// Built-in layout used until real stylesheets can be loaded.
    pub fn builtin_magic_placeholder() -> Self {
        let f = |field: &str, l, t, w, h| FieldStyle { field: field.into(), left: l, top: t, width: w, height: h };
        let mut stylesheet = StyleSheet::default_for("placeholder");
        stylesheet.styles = vec![
            f("name", 25.0, 22.0, 325.0, 30.0),
            f("casting_cost", 250.0, 22.0, 100.0, 30.0),
            f("image", 30.0, 60.0, 315.0, 230.0),
            f("type_line", 25.0, 295.0, 325.0, 28.0),
            f("rule_text", 30.0, 330.0, 315.0, 130.0),
            f("pt", 275.0, 480.0, 70.0, 30.0),
        ];
        let game = Game {
            name: "magic".into(),
            card_fields: ["name", "casting_cost", "image", "type_line", "rule_text", "pt"]
                .iter()
                .map(|n| FieldDef { name: (*n).into(), kind: "text".into() })
                .collect(),
        };
        Template { game, stylesheet }
    }
}
