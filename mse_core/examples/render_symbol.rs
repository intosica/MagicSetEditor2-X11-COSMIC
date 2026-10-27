//! cargo run -p mse_core --example render_symbol -- in.mse-symbol out.png [size] [border_radius]
use mse_core::symbol::{Symbol, SymbolStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: render_symbol <in.mse-symbol> <out.png> [size=512] [border_radius=0.03]");
        std::process::exit(2);
    }
    let size: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(512);
    let radius: f32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.03);
    let r = Symbol::load(&a[1])?.render(size, SymbolStyle { border_radius: radius, ..Default::default() })?;

    // composite over mid-grey so both the white fill and black outline are visible
    let mut out = vec![255u8; r.rgba.len()];
    for (o, p) in out.chunks_exact_mut(4).zip(r.rgba.chunks_exact(4)) {
        let al = p[3] as f32 / 255.0;
        for c in 0..3 {
            o[c] = (p[c] as f32 * al + 128.0 * (1.0 - al)).round() as u8;
        }
    }
    image::RgbaImage::from_raw(r.width, r.height, out).ok_or("size mismatch")?.save(&a[2])?;
    println!("wrote {}", a[2]);
    Ok(())
}
