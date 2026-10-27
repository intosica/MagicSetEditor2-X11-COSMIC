//! Set-symbol rendering.
//!
//! ## Why this differs from the C++ implementation
//! Legacy MSE draws each shape twice (a thick white pen stroke = "border" and
//! a filled polygon = "interior") into two 1-bit-style DCs and merges them with
//! raster ops (`wxOR` / `wxAND_INVERT` blits). On GTK3/Cairo wx builds (e.g. the
//! Flatpak) those logical functions are not honoured, so the "border minus
//! interior" step never happens and the border blob covers the whole symbol.
//!
//! Here both layers are plain 8-bit coverage masks and the final ring is
//! computed explicitly: `ring = border * (1 - interior)`. No raster ops.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    Merge,
    Subtract,
    Intersection,
    Difference,
    Overlap,
    Border,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentMode {
    Line,
    Curve,
}

/// Coordinates are in symbol space, 0.0..=1.0 on both axes.
#[derive(Debug, Clone, Copy)]
pub struct ControlPoint {
    pub pos: (f32, f32),
    /// Bezier handle offsets relative to `pos`.
    pub handle_before: (f32, f32),
    pub handle_after: (f32, f32),
    /// Mode of the segment leaving this point.
    pub mode: SegmentMode,
}

impl ControlPoint {
    pub fn line(x: f32, y: f32) -> Self {
        Self { pos: (x, y), handle_before: (0.0, 0.0), handle_after: (0.0, 0.0), mode: SegmentMode::Line }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolShape {
    pub name: String,
    pub points: Vec<ControlPoint>,
    pub combine: Combine,
}

#[derive(Debug, Clone, Default)]
pub struct Symbol {
    /// Bottom-to-top draw order.
    pub shapes: Vec<SymbolShape>,
    // TODO: symmetry groups, .mse-symbol parsing.
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolStyle {
    /// Border width as a fraction of symbol size (MSE default is ~0.05..0.1).
    pub border_radius: f32,
    pub fill: [u8; 3],
    pub border: [u8; 3],
}

impl Default for SymbolStyle {
    fn default() -> Self {
        Self { border_radius: 0.06, fill: [255, 255, 255], border: [0, 0, 0] }
    }
}

pub struct RenderedSymbol {
    pub width: u32,
    pub height: u32,
    /// Straight (non-premultiplied) RGBA.
    pub rgba: Vec<u8>,
    /// Raw masks, exposed for tests / custom colouring.
    pub interior: Vec<u8>,
    pub ring: Vec<u8>,
}

impl SymbolShape {
    fn path(&self, size: f32) -> Option<tiny_skia::Path> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }
        let s = |p: (f32, f32)| (p.0 * size, p.1 * size);
        let mut pb = PathBuilder::new();
        let (x0, y0) = s(self.points[0].pos);
        pb.move_to(x0, y0);
        for i in 0..n {
            let a = &self.points[i];
            let b = &self.points[(i + 1) % n];
            let (bx, by) = s(b.pos);
            match a.mode {
                SegmentMode::Line => pb.line_to(bx, by),
                SegmentMode::Curve => {
                    let (c1x, c1y) = s((a.pos.0 + a.handle_after.0, a.pos.1 + a.handle_after.1));
                    let (c2x, c2y) = s((b.pos.0 + b.handle_before.0, b.pos.1 + b.handle_before.1));
                    pb.cubic_to(c1x, c1y, c2x, c2y, bx, by);
                }
            }
        }
        pb.close();
        pb.finish()
    }
}

fn coverage(pm: &Pixmap) -> Vec<u8> {
    pm.data().chunks_exact(4).map(|p| p[3]).collect()
}

impl Symbol {
    pub fn render(&self, size: u32, style: SymbolStyle) -> Result<RenderedSymbol> {
        let n = (size * size) as usize;
        let sf = size as f32;
        let stroke_w = (style.border_radius * sf).max(0.0);
        let mut border = vec![0u8; n]; // union of thickened shapes
        let mut interior = vec![0u8; n]; // union of plain shapes

        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        paint.anti_alias = true;

        for shape in &self.shapes {
            let Some(path) = shape.path(sf) else { continue };
            let mut fill_pm = Pixmap::new(size, size).ok_or_else(|| Error::Render("bad symbol size".into()))?;
            fill_pm.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            let f = coverage(&fill_pm);

            // thickened = fill + centred stroke
            let mut thick_pm = fill_pm.clone();
            if stroke_w > 0.0 {
                let stroke = Stroke { width: stroke_w, ..Stroke::default() };
                thick_pm.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            let t = coverage(&thick_pm);

            match shape.combine {
                Combine::Merge | Combine::Overlap => {
                    zip_apply(&mut border, &t, |d, s| d.max(s));
                    zip_apply(&mut interior, &f, |d, s| d.max(s));
                }
                Combine::Subtract => {
                    zip_apply(&mut border, &f, |d, s| mul(d, 255 - s));
                    zip_apply(&mut interior, &f, |d, s| mul(d, 255 - s));
                }
                Combine::Intersection => {
                    zip_apply(&mut border, &t, mul);
                    zip_apply(&mut interior, &f, mul);
                }
                Combine::Difference => {
                    zip_apply(&mut interior, &f, |d, s| d.max(s) - mul(d, s)); // xor-ish
                    zip_apply(&mut border, &t, |d, s| d.max(s));
                }
                Combine::Border => {
                    // shape's own ring becomes interior
                    let ring: Vec<u8> = t.iter().zip(&f).map(|(t, f)| mul(*t, 255 - *f)).collect();
                    zip_apply(&mut interior, &ring, |d, s| d.max(s));
                }
            }
        }

        // The fix: explicit border-minus-interior.
        let ring: Vec<u8> = border.iter().zip(&interior).map(|(b, i)| mul(*b, 255 - *i)).collect();

        let mut rgba = vec![0u8; n * 4];
        for (k, px) in rgba.chunks_exact_mut(4).enumerate() {
            let ai = interior[k] as f32 / 255.0;
            let ar = ring[k] as f32 / 255.0;
            let a = (ai + ar).min(1.0);
            if a > 0.0 {
                for c in 0..3 {
                    px[c] = ((style.fill[c] as f32 * ai + style.border[c] as f32 * ar) / (ai + ar)).round() as u8;
                }
                px[3] = (a * 255.0).round() as u8;
            }
        }
        Ok(RenderedSymbol { width: size, height: size, rgba, interior, ring })
    }
}

fn zip_apply(dst: &mut [u8], src: &[u8], f: impl Fn(u8, u8) -> u8) {
    for (d, s) in dst.iter_mut().zip(src) {
        *d = f(*d, *s);
    }
}
fn mul(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16 + 127) / 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Symbol {
        let pts = [(0.3, 0.3), (0.7, 0.3), (0.7, 0.7), (0.3, 0.7)];
        Symbol {
            shapes: vec![SymbolShape {
                name: "sq".into(),
                points: pts.iter().map(|p| ControlPoint::line(p.0, p.1)).collect(),
                combine: Combine::Merge,
            }],
        }
    }

    /// Regression test for the Flatpak bug: border must be an OUTLINE only.
    #[test]
    fn border_is_a_ring_not_a_blob() {
        let r = square().render(100, SymbolStyle { border_radius: 0.1, ..Default::default() }).unwrap();
        let at = |m: &[u8], x: usize, y: usize| m[y * 100 + x];
        assert_eq!(at(&r.interior, 50, 50), 255, "centre is interior");
        assert_eq!(at(&r.ring, 50, 50), 0, "centre must NOT be border");
        assert!(at(&r.ring, 27, 50) > 200, "just outside the edge is border");
        assert_eq!(at(&r.ring, 5, 50), 0, "far away is empty");
    }
}
