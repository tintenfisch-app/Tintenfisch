use crate::support;

use support::rendering::TestRenderer;
use support::synthetic_pdf::{Row, SyntheticPdf};

fn showing(subtype: &str, base_font: &str, width: u32, text: &str) -> Vec<u8> {
    let widths = vec![width.to_string(); 95].join(" ");
    let font = format!(
        "<< /Type /Font /Subtype /{subtype} /BaseFont /{base_font} /Encoding /WinAnsiEncoding /FirstChar 32 /LastChar 126 /Widths [{widths}] >>"
    );
    let resources = "/Resources << /Font << /F1 5 0 R >> >>";
    let page = format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 150] /Contents 4 0 R {resources} >>");
    let content = format!("BT /F1 100 Tf 20 30 Td ({text}) Tj ET");
    let mut pdf = SyntheticPdf::new("1.4");
    pdf.object(1, 0, "<< /Type /Catalog /Pages 2 0 R >>")
        .object(2, 0, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
        .object(3, 0, &page)
        .stream(4, "", content.as_bytes())
        .object(5, 0, &font);
    let rows: Vec<(u32, Row)> =
        std::iter::once((0, Row::Free(65535))).chain((1..=5).map(|num| (num, Row::InUse(num)))).collect();
    let start = pdf.xref_table(&rows, "/Size 6 /Root 1 0 R");
    pdf.startxref(start).finish()
}

fn inked_columns(bytes: Vec<u8>) -> usize {
    let raster = TestRenderer::new(bytes).render(0, 1.0, None);
    (0..raster.width as usize)
        .filter(|&x| (0..raster.height as usize).any(|y| raster.rgba[(y * raster.width as usize + x) * 4] < 128))
        .count()
}

#[test]
fn a_wide_substitute_glyph_is_narrowed_to_its_declared_width() {
    let inked = inked_columns(showing("TrueType", "ComicSansMS", 500, "m"));
    assert!((30..=50).contains(&inked), "an m fits its 50 point cell, inked {inked}");
}

#[test]
fn a_narrow_substitute_glyph_is_widened_toward_its_declared_width_within_limits() {
    let natural = inked_columns(showing("Type1", "Garamond-Bold", 278, "i")) as f64;
    let declared = inked_columns(showing("Type1", "Garamond-Bold", 500, "i")) as f64;
    let extreme = inked_columns(showing("Type1", "Garamond-Bold", 5000, "i")) as f64;
    assert!((1.5..=2.0).contains(&(declared / natural)), "widened {natural} to {declared}");
    assert!(extreme <= natural * 2.2, "an absurd width stays capped: {natural} to {extreme}");
}
