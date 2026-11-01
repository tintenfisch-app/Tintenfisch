use std::io::Write;
use std::rc::Rc;

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::error::{Error, Result};
use crate::image::{DecodedImage, JpegHeader};
use crate::store::Transaction;
use crate::syntax::{Dict, Name, ObjId, Object, Stream};

pub struct ImageStreams {
    pub image: Stream,
    pub soft_mask: Option<Stream>,
}

impl ImageStreams {
    pub fn rgb(decoded: &DecodedImage) -> Result<ImageStreams> {
        let rows = || decoded.rgba.as_chunks::<4>().0.chunks(decoded.width.max(1));
        let samples = |space: &str| {
            vec![
                ("ColorSpace", Object::Name(Name::from(space))),
                ("BitsPerComponent", Object::Int(8)),
                ("Filter", Object::Name(Name::from("FlateDecode"))),
            ]
        };
        let color =
            deflated(rows().map(|row| row.iter().flat_map(|[red, green, blue, _]| [*red, *green, *blue]).collect()))?;
        let translucent = decoded.rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] < 255);
        let soft_mask = if translucent {
            let alpha = deflated(rows().map(|row| row.iter().map(|pixel| pixel[3]).collect()))?;
            Some(Stream { dict: image_dict(decoded.width, decoded.height, samples("DeviceGray")), raw: alpha })
        } else {
            None
        };
        Ok(ImageStreams {
            image: Stream { dict: image_dict(decoded.width, decoded.height, samples("DeviceRGB")), raw: color },
            soft_mask,
        })
    }

    pub fn stencil(decoded: &DecodedImage) -> ImageStreams {
        let (width, height) = (decoded.width, decoded.height);
        let row_bytes = width.div_ceil(8);
        let mut bits = vec![0u8; row_bytes * height];
        for y in 0..height {
            for x in 0..width {
                if decoded.rgba[(y * width + x) * 4 + 3] < 128 {
                    bits[y * row_bytes + x / 8] |= 0x80 >> (x % 8);
                }
            }
        }
        let dict = image_dict(
            decoded.width,
            decoded.height,
            vec![("ImageMask", Object::Bool(true)), ("BitsPerComponent", Object::Int(1))],
        );
        ImageStreams { image: Stream { dict, raw: Rc::from(bits) }, soft_mask: None }
    }

    pub fn jpeg(header: &JpegHeader, data: &[u8]) -> ImageStreams {
        let space = match header.components {
            1 => "DeviceGray",
            4 => "DeviceCMYK",
            _ => "DeviceRGB",
        };
        let mut extra = vec![
            ("ColorSpace", Object::Name(Name::from(space))),
            ("BitsPerComponent", Object::Int(8)),
            ("Filter", Object::Name(Name::from("DCTDecode"))),
        ];
        if header.components == 4 && header.adobe {
            extra.push(("Decode", Object::Array([1, 0, 1, 0, 1, 0, 1, 0].map(Object::Int).to_vec())));
        }
        let dict = image_dict(header.width, header.height, extra);
        ImageStreams { image: Stream { dict, raw: Rc::from(data) }, soft_mask: None }
    }

    pub fn allocate(&self, transaction: &mut Transaction) -> ObjId {
        let mut image = self.image.clone();
        if let Some(mask) = &self.soft_mask {
            image.dict.insert("SMask", Object::Ref(transaction.allocate(Object::Stream(mask.clone()))));
        }
        transaction.allocate(Object::Stream(image))
    }
}

fn deflated(rows: impl Iterator<Item = Vec<u8>>) -> Result<Rc<[u8]>> {
    let failed = |error: std::io::Error| Error::new(format!("image compression: {error}"));
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    for row in rows {
        encoder.write_all(&row).map_err(failed)?;
    }
    Ok(Rc::from(encoder.finish().map_err(failed)?))
}

fn image_dict(width: usize, height: usize, extra: Vec<(&str, Object)>) -> Dict {
    [
        ("Type", Object::Name(Name::from("XObject"))),
        ("Subtype", Object::Name(Name::from("Image"))),
        ("Width", Object::Int(width as i64)),
        ("Height", Object::Int(height as i64)),
    ]
    .into_iter()
    .chain(extra)
    .map(|(key, value)| (Name::from(key), value))
    .collect()
}
