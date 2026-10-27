//! End-to-end check with a real symbol saved from the MSE symbol editor.

use mse_core::symbol::{Symbol, SymbolStyle};

fn load() -> Symbol {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/Intosica.mse-symbol");
    Symbol::load(p).expect("symbol loads")
}

#[test]
fn loads_three_shapes() {
    let s = load();
    assert_eq!(s.shapes.len(), 3);
    assert_eq!(s.shapes.iter().map(|s| s.points.len()).sum::<usize>(), 73);
}

#[test]
fn outline_is_hollow() {
    let size = 400u32;
    let r = load().render(size, SymbolStyle { border_radius: 0.03, ..Default::default() }).unwrap();
    let at = |m: &[u8], x: f32, y: f32| m[(y * size as f32) as usize * size as usize + (x * size as f32) as usize];
    // (0.39, 0.775) lies inside the lowest tadpole's body.
    assert_eq!(at(&r.interior, 0.39, 0.775), 255);
    assert_eq!(at(&r.ring, 0.39, 0.775), 0, "body interior must not be border-coloured");
    // The outline exists, but covers far less area than the fill.
    let ring: usize = r.ring.iter().filter(|v| **v > 127).count();
    let fill: usize = r.interior.iter().filter(|v| **v > 127).count();
    assert!(ring > 0 && ring < fill, "ring={ring} fill={fill}");
    // no pixel is fully both
    assert!(r.ring.iter().zip(&r.interior).all(|(r, i)| (*r as u16 + *i as u16) <= 256));
}
