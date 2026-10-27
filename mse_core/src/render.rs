//! Card composition. Placeholder renderer: frame + field boxes.
//! TODO: real text layout (cosmic-text), mana symbol fonts, image fitting,
//! stylesheet scripts, layered card frames.

use std::path::Path;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::{card::Card, error::Error, template::Template, Result};

pub struct RenderedCard {
    pub width: u32,
    pub height: u32,
    /// RGBA8, opaque.
    pub rgba: Vec<u8>,
}

impl RenderedCard {
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let img = image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| Error::Render("buffer size mismatch".into()))?;
        img.save_with_format(path, image::ImageFormat::Png)?;
        Ok(())
    }
}

pub fn render_card(card: &Card, template: &Template) -> Result<RenderedCard> {
    let (w, h) = (template.stylesheet.card_width, template.stylesheet.card_height);
    let mut pm = Pixmap::new(w, h).ok_or_else(|| Error::Render("zero-sized card".into()))?;
    pm.fill(tiny_skia::Color::from_rgba8(236, 226, 205, 255));

    let mut frame = Paint::default();
    frame.set_color_rgba8(40, 40, 40, 255);
    frame.anti_alias = true;
    let mut boxfill = Paint::default();
    boxfill.set_color_rgba8(250, 247, 240, 255);

    for st in &template.stylesheet.styles {
        let Some(r) = Rect::from_xywh(st.left, st.top, st.width, st.height) else { continue };
        let path = PathBuilder::from_rect(r);
        pm.fill_path(&path, &boxfill, FillRule::Winding, Transform::identity(), None);
        pm.stroke_path(&path, &frame, &Stroke { width: 1.5, ..Stroke::default() }, Transform::identity(), None);
        // TODO: draw card.get(&st.field) as text
        let _ = card.get(&st.field);
    }
    Ok(RenderedCard { width: w, height: h, rgba: pm.take() })
}
