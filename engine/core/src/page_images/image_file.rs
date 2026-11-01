use crate::error::{Error, Result};
use crate::geometry::Matrix;
use crate::image::{DecodedImage, ImageStreams, JpegHeader, decode_png, png_size};

const POINTS_PER_PIXEL: f64 = 0.75;
const MOST_SOURCE_MEGAPIXELS: usize = 50;
const MOST_EMBEDDED_SIDE: usize = 4096;
const MOST_EMBEDDED_PIXELS: usize = 1 << 22;

pub struct ImageFile {
    pub streams: ImageStreams,
    pub orientation: Matrix,
    pub natural_size: (f64, f64),
}

impl ImageFile {
    pub fn read(data: &[u8]) -> Result<ImageFile> {
        let (streams, (width, height), orientation) = if data.starts_with(b"\x89PNG") {
            let (width, height) = png_size(data)?;
            refuse_oversized(width, height)?;
            let decoded = bounded(decode_png(data)?);
            (ImageStreams::rgb(&decoded)?, (width, height), 1)
        } else if data.starts_with(&[0xFF, 0xD8]) {
            let header = JpegHeader::read(data)?;
            refuse_oversized(header.width, header.height)?;
            (ImageStreams::jpeg(&header, data), (header.width, header.height), header.orientation)
        } else {
            return Err(Error::unsupported("only PNG and JPEG images can be placed"));
        };
        let turned = orientation >= 5;
        let (shown_width, shown_height) = if turned { (height, width) } else { (width, height) };
        Ok(ImageFile {
            streams,
            orientation: oriented(orientation),
            natural_size: (shown_width as f64 * POINTS_PER_PIXEL, shown_height as f64 * POINTS_PER_PIXEL),
        })
    }

    pub fn aspect(&self) -> f64 {
        self.natural_size.0 / self.natural_size.1
    }
}

fn refuse_oversized(width: usize, height: usize) -> Result<()> {
    if width.saturating_mul(height) <= MOST_SOURCE_MEGAPIXELS * 1_000_000 {
        return Ok(());
    }
    Err(Error::unsupported(format!(
        "this image has {width} by {height} pixels, images up to {MOST_SOURCE_MEGAPIXELS} megapixels can be placed"
    )))
}

fn bounded(decoded: DecodedImage) -> DecodedImage {
    let fits = |factor: usize| {
        let (width, height) = (decoded.width.div_ceil(factor), decoded.height.div_ceil(factor));
        width.max(height) <= MOST_EMBEDDED_SIDE && width * height <= MOST_EMBEDDED_PIXELS
    };
    match (1..).find(|factor| fits(*factor)) {
        Some(factor) if factor > 1 => decoded.reduced(factor, factor),
        _ => decoded,
    }
}

fn oriented(orientation: u16) -> Matrix {
    match orientation {
        2 => Matrix::new(-1.0, 0.0, 0.0, 1.0, 1.0, 0.0),
        3 => Matrix::new(-1.0, 0.0, 0.0, -1.0, 1.0, 1.0),
        4 => Matrix::new(1.0, 0.0, 0.0, -1.0, 0.0, 1.0),
        5 => Matrix::new(0.0, -1.0, -1.0, 0.0, 1.0, 1.0),
        6 => Matrix::new(0.0, -1.0, 1.0, 0.0, 0.0, 1.0),
        7 => Matrix::new(0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
        8 => Matrix::new(0.0, 1.0, -1.0, 0.0, 1.0, 0.0),
        _ => Matrix::IDENTITY,
    }
}
