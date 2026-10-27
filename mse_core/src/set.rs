use std::{collections::HashMap, fs::File, io::Read, path::Path};

use crate::{card::Card, error::Error, mse_format, Result};

/// A loaded `.mse-set` file (a zip archive whose `set` entry holds the data).
#[derive(Debug, Clone, Default)]
pub struct Set {
    pub game: String,
    pub stylesheet: String,
    /// `set info:` key/values (title, copyright, ...).
    pub info: HashMap<String, String>,
    pub cards: Vec<Card>,
    /// Every other archive member (card art, symbols, ...), by archive name.
    pub assets: HashMap<String, Vec<u8>>,
}

impl Set {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| Error::Io { path: path.into(), source })?;
        Self::from_reader(file)
    }

    pub fn from_reader<R: Read + std::io::Seek>(reader: R) -> Result<Self> {
        let mut zip = zip::ZipArchive::new(reader)?;
        let mut set_text = None;
        let mut assets = HashMap::new();
        for i in 0..zip.len() {
            let mut f = zip.by_index(i)?;
            if f.is_dir() {
                continue;
            }
            let name = f.name().to_owned();
            let mut buf = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buf).map_err(|source| Error::Io { path: name.clone().into(), source })?;
            if name == "set" {
                set_text = Some(String::from_utf8(buf)?);
            } else {
                assets.insert(name, buf);
            }
        }
        let text = set_text.ok_or(Error::MissingSetFile)?;
        let mut set = Self::from_text(&text)?;
        set.assets = assets;
        Ok(set)
    }

    pub fn from_text(text: &str) -> Result<Self> {
        let nodes = mse_format::parse(text)?;
        let mut set = Set::default();
        for n in &nodes {
            match n.key.as_str() {
                "game" => set.game = n.text(),
                "stylesheet" => set.stylesheet = n.text(),
                "set info" => {
                    for c in &n.children {
                        set.info.insert(c.key.clone(), c.text());
                    }
                }
                "card" => set.cards.push(Card::from_node(n)),
                _ => {} // styling, version, etc. – TODO
            }
        }
        Ok(set)
    }

    // TODO: Set::save() – write `set` text + assets back into a zip.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cards() {
        let s = Set::from_text("game: magic\nstylesheet: m15\ncard:\n\tname: A\ncard:\n\tname: B\n").unwrap();
        assert_eq!(s.game, "magic");
        assert_eq!(s.cards.len(), 2);
        assert_eq!(s.cards[1].name(), "B");
    }
}
