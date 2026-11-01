use std::fs;
use std::ops::Range;
use std::path::PathBuf;

use crate::support::rendering;
use crate::support::synthetic_pdf::{Row, SyntheticPdf};
use tintenfisch::document::Document;
use tintenfisch::fonts::FontLibrary;
use tintenfisch::render::Raster;

const CONTENT: &[u8] = b"BT /F1 24 Tf 20 150 Td <4E2D65875B57> Tj ET \
BT /F2 24 Tf 20 100 Td <65E5672C8A9E> Tj ET \
BT /F3 20 Tf 20 40 Td <0102030405> Tj ET \
BT /F4 24 Tf 200 150 Td <0001> Tj ET";

const PRESENTATION_FORMS: &[u8] = b"begincmap 1 begincodespacerange <00> <FF> endcodespacerange \
5 beginbfchar <01> <FE8E> <02> <FE92> <03> <FEA3> <04> <FEAE> <05> <FEE3> endbfchar endcmap";

const IDEOGRAPH: &[u8] = b"begincmap 1 begincodespacerange <0000> <FFFF> endcodespacerange \
1 beginbfchar <0001> <4E2D> endbfchar endcmap";

fn cid_font(base_font: &str, cmap: &str, ordering: &str, extra: &str) -> String {
    format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /{base_font} /Encoding /{cmap} {extra} /DescendantFonts [<< /Type /Font \
         /Subtype /CIDFontType0 /BaseFont /{base_font} /CIDSystemInfo << /Registry (Adobe) /Ordering ({ordering}) \
         /Supplement 2 >> /FontDescriptor << /Type /FontDescriptor /FontName /{base_font} /Flags 6 \
         /FontBBox [0 -120 1000 880] /ItalicAngle 0 /Ascent 880 /Descent -120 /CapHeight 700 /StemV 80 >> \
         /DW 1000 /W [34 [600]] >>] >>"
    )
}

fn multilingual_page() -> Vec<u8> {
    let mut pdf = SyntheticPdf::new("1.6");
    pdf.object(1, 0, "<< /Type /Catalog /Pages 2 0 R >>")
        .object(2, 0, "<< /Type /Pages /Kids [3 0 R 7 0 R] /Count 2 >>")
        .object(
            3,
            0,
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Contents 4 0 R \
             /Resources << /Font << /F1 5 0 R /F2 6 0 R /F3 8 0 R /F4 11 0 R >> >> >>",
        )
        .stream(4, "", CONTENT)
        .object(5, 0, &cid_font("STSong-Light", "UniGB-UCS2-H", "GB1", ""))
        .object(6, 0, &cid_font("HeiseiMin-W3", "UniJIS-UCS2-H", "Japan1", ""))
        .object(
            7,
            0,
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Contents 9 0 R \
             /Resources << /Font << /F3 8 0 R >> >> >>",
        )
        .object(8, 0, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /ToUnicode 10 0 R >>")
        .stream(9, "", b"BT /F3 20 Tf 20 40 Td <0102030405> Tj ET")
        .stream(10, "", PRESENTATION_FORMS)
        .object(11, 0, &cid_font("MS-Mincho", "Identity-H", "Japan1", "/ToUnicode 12 0 R"))
        .stream(12, "", IDEOGRAPH);
    let rows: Vec<(u32, Row)> =
        std::iter::once((0, Row::Free(65535))).chain((1..=12).map(|num| (num, Row::InUse(num)))).collect();
    let start = pdf.xref_table(&rows, "/Size 13 /Root 1 0 R");
    pdf.startxref(start).finish()
}

fn with_cjk_face() -> FontLibrary {
    let mut library = rendering::library();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../public/fonts").join(FontLibrary::CJK_FILE_NAME);
    library.register_cjk(fs::read(path).unwrap());
    library
}

fn inked_pixels(raster: &Raster, columns: Range<u32>, rows: Range<u32>) -> usize {
    rows.flat_map(|y| columns.clone().map(move |x| ((y * raster.width + x) * 4) as usize))
        .filter(|&index| raster.rgba[index] < 128)
        .count()
}

fn lines(document: &Document, library: &FontLibrary, page: usize) -> Vec<String> {
    let view = document.view(library);
    let layout = view.text_layout(view.pages().unwrap()[page].id).unwrap();
    layout.lines().iter().map(|line| line.clusters.iter().map(|cluster| cluster.text.as_str()).collect()).collect()
}

#[test]
fn predefined_unicode_cmaps_extract_and_find_their_characters() {
    let document = Document::open(multilingual_page(), "").unwrap();
    let library = rendering::library();
    let lines = lines(&document, &library, 0);
    assert!(lines.iter().any(|line| line.starts_with("中文字")), "{lines:?}");
    assert!(lines.iter().any(|line| line == "日本語"), "{lines:?}");
    let view = document.view(&library);
    let layout = view.text_layout(view.pages().unwrap()[0].id).unwrap();
    assert_eq!(layout.find("中").len(), 2);
    assert_eq!(layout.find("本語").len(), 1);
}

#[test]
fn arabic_presentation_forms_are_found_by_their_letters() {
    let document = Document::open(multilingual_page(), "").unwrap();
    let library = rendering::library();
    assert_eq!(lines(&document, &library, 1), ["ابحرم"]);
    let view = document.view(&library);
    let layout = view.text_layout(view.pages().unwrap()[1].id).unwrap();
    assert_eq!(layout.find("مرحبا").len(), 1);
    assert_eq!(layout.find("\u{FEE3}\u{FEAE}\u{FEA3}\u{FE92}\u{FE8E}").len(), 1);
    assert!(layout.find("مرحبات").is_empty());
}

#[test]
fn non_embedded_cjk_fonts_render_once_the_bundled_face_arrives() {
    let document = Document::open(multilingual_page(), "").unwrap();
    let latin = rendering::library();
    let pages = document.view(&latin).pages().unwrap();
    assert!(document.view(&latin).needs_cjk_substitute(pages[0].id).unwrap());
    assert!(!document.view(&latin).needs_cjk_substitute(pages[1].id).unwrap());

    let blank = document.view(&latin).render(pages[0].id, 1.0, None).unwrap();
    assert_eq!(inked_pixels(&blank, 0..300, 20..110), 0);

    let cjk = with_cjk_face();
    document.forget_fonts();
    let rendered = document.view(&cjk).render(pages[0].id, 1.0, None).unwrap();
    assert!(inked_pixels(&rendered, 0..100, 20..55) > 100, "the Chinese line is drawn");
    assert!(inked_pixels(&rendered, 0..100, 70..105) > 100, "the Japanese line is drawn");
    assert!(
        inked_pixels(&rendered, 195..230, 20..55) > 30,
        "the identity coded ideograph is drawn by its Unicode value"
    );
}
