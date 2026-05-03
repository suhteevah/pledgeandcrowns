// Tiny standalone helper: compare two PNGs byte-decoded as RGBA8 and
// report whether every pixel matches. Used for the legacy-pipeline
// parity check; not part of the main render binary.

use anyhow::{Context, Result, bail};
use image::GenericImageView;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let a = args.next().context("usage: compare_png <a.png> <b.png>")?;
    let b = args.next().context("usage: compare_png <a.png> <b.png>")?;

    let img_a = image::open(&a).with_context(|| format!("opening {a}"))?;
    let img_b = image::open(&b).with_context(|| format!("opening {b}"))?;

    if img_a.dimensions() != img_b.dimensions() {
        bail!(
            "dimension mismatch: {:?} vs {:?}",
            img_a.dimensions(),
            img_b.dimensions()
        );
    }

    let rgba_a = img_a.to_rgba8();
    let rgba_b = img_b.to_rgba8();
    let mut diffs = 0usize;
    for (pa, pb) in rgba_a.pixels().zip(rgba_b.pixels()) {
        if pa.0 != pb.0 {
            diffs += 1;
        }
    }
    let total = (img_a.width() * img_a.height()) as usize;
    println!(
        "{} vs {}: {}/{} pixels differ ({:.2}%)",
        a,
        b,
        diffs,
        total,
        100.0 * diffs as f64 / total as f64
    );
    Ok(())
}
