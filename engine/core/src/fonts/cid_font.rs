use std::rc::Rc;

use tiny_skia::Path;

use crate::error::{Error, Result};
use crate::fonts::{CidCmap, FallbackFace, FontDescriptor, FontLibrary, FontProgram};
use crate::store::ObjectStore;
use crate::syntax::{Dict, Object};

const CJK_ORDERINGS: [&[u8]; 6] = [b"GB1", b"CNS1", b"Japan1", b"Japan2", b"Korea1", b"KR"];

enum CidToGid {
    Identity,
    Table(Vec<u16>),
}

pub struct CidFont {
    program: Option<FontProgram>,
    embedded: bool,
    cmap: CidCmap,
    cid_to_gid: CidToGid,
    default_width: f64,
    widths: Vec<(u32, u32, f64)>,
}

impl CidFont {
    pub fn load(font: &Dict, store: &ObjectStore, library: &FontLibrary) -> Result<CidFont> {
        let descendant = descendant(font, store)?;
        let cmap = match font.get("Encoding").map(|encoding| store.resolve(encoding)).transpose()? {
            Some(Object::Name(name)) => CidCmap::predefined(&name),
            Some(Object::Stream(stream)) => CidCmap::parse(&store.decode_stream(&stream)?.data),
            _ => CidCmap::identity(false),
        };
        let descriptor = FontDescriptor::read(&descendant, store)?;
        let embedded = descriptor.program.is_some();
        let program = match descriptor.program {
            Some(program) => Some(program),
            None => {
                let base_font = font
                    .name("BaseFont")
                    .map(|name| String::from_utf8_lossy(name.as_bytes()).into_owned())
                    .unwrap_or_default();
                let cjk = is_cjk_collection(&descendant, store).then(|| library.cjk_face()).flatten();
                cjk.or_else(|| library.face(FallbackFace::for_font(&base_font, descriptor.flags)))
                    .and_then(FontProgram::parse)
            }
        };
        let cid_to_gid = match descendant.get("CIDToGIDMap").map(|map| store.resolve(map)).transpose()? {
            Some(Object::Stream(stream)) => {
                let data = store.decode_stream(&stream)?.data;
                CidToGid::Table(data.as_chunks::<2>().0.iter().map(|&pair| u16::from_be_bytes(pair)).collect())
            }
            _ => CidToGid::Identity,
        };
        let default_width = descendant.get("DW").and_then(Object::as_number).unwrap_or(1000.0) / 1000.0;
        let widths = read_widths(&descendant, store)?;
        Ok(CidFont { program, embedded, cmap, cid_to_gid, default_width, widths })
    }

    pub fn needs_cjk_substitute(font: &Dict, store: &ObjectStore) -> bool {
        if !font.name("Subtype").is_some_and(|subtype| subtype.is("Type0")) {
            return false;
        }
        let Ok(descendant) = descendant(font, store) else { return false };
        let embedded = descendant
            .get("FontDescriptor")
            .and_then(|descriptor| store.resolve(descriptor).ok())
            .and_then(|descriptor| descriptor.as_dict().cloned())
            .is_some_and(|descriptor| {
                ["FontFile", "FontFile2", "FontFile3"].iter().any(|key| descriptor.get(key).is_some())
            });
        !embedded && is_cjk_collection(&descendant, store)
    }

    pub fn next_code(&self, bytes: &[u8]) -> (u32, usize) {
        self.cmap.next_code(bytes)
    }

    pub fn cid(&self, code: u32, length: usize) -> u32 {
        self.cmap.cid(code, length)
    }

    pub fn glyph(&self, cid: u32, substitute_character: Option<char>) -> Option<u16> {
        if let Some(character) = substitute_character.filter(|_| !self.embedded) {
            return self.program.as_ref()?.glyph_for_char(character);
        }
        let cid = u16::try_from(cid).ok()?;
        match &self.cid_to_gid {
            CidToGid::Table(table) => table.get(cid as usize).copied(),
            CidToGid::Identity => self.program.as_ref()?.glyph_for_cid(cid),
        }
    }

    pub fn width(&self, cid: u32) -> f64 {
        self.widths
            .iter()
            .find(|(first, last, _)| (*first..=*last).contains(&cid))
            .map_or(self.default_width, |(_, _, width)| *width)
    }

    pub fn unicode(&self, code: u32) -> Option<char> {
        self.cmap.unicode(code)
    }

    pub fn code_for(&self, character: char) -> Option<(u32, usize)> {
        if self.cmap.is_unicode() {
            return self.cmap.code_for(character);
        }
        if !self.cmap.is_identity() || !matches!(self.cid_to_gid, CidToGid::Identity) {
            return None;
        }
        let glyph = self.program.as_ref()?.glyph_for_char(character)?;
        Some((glyph as u32, 2))
    }

    pub fn is_vertical(&self) -> bool {
        self.cmap.vertical
    }

    pub fn outline(&self, glyph: u16) -> Option<Rc<Path>> {
        self.program.as_ref()?.outline(glyph)
    }

    pub fn embedded_program(&self) -> Option<&FontProgram> {
        self.program.as_ref().filter(|_| self.embedded)
    }
}

fn descendant(font: &Dict, store: &ObjectStore) -> Result<Dict> {
    let descendants = store.resolve(font.get("DescendantFonts").unwrap_or(&Object::Null))?;
    descendants
        .as_array()
        .and_then(|items| items.first())
        .map(|first| store.resolve(first))
        .transpose()?
        .and_then(|descendant| descendant.as_dict().cloned())
        .ok_or_else(|| Error::new("Type0 font without a descendant font"))
}

fn is_cjk_collection(descendant: &Dict, store: &ObjectStore) -> bool {
    let Some(info) = descendant
        .get("CIDSystemInfo")
        .and_then(|info| store.resolve(info).ok())
        .and_then(|info| info.as_dict().cloned())
    else {
        return false;
    };
    let ordering = match info.get("Ordering").and_then(|ordering| store.resolve(ordering).ok()) {
        Some(Object::String(bytes)) => bytes,
        Some(Object::Name(name)) => name.as_bytes().to_vec(),
        _ => return false,
    };
    CJK_ORDERINGS.contains(&ordering.as_slice())
}

fn read_widths(descendant: &Dict, store: &ObjectStore) -> Result<Vec<(u32, u32, f64)>> {
    let listed = store.resolve(descendant.get("W").unwrap_or(&Object::Null))?;
    let items = listed.as_array().unwrap_or_default();
    let mut widths = Vec::new();
    let mut index = 0;
    while index < items.len() {
        let Some(first) = items[index].as_int().map(|first| first.max(0) as u32) else {
            index += 1;
            continue;
        };
        match store.resolve(items.get(index + 1).unwrap_or(&Object::Null))? {
            Object::Array(list) => {
                for (offset, width) in list.iter().enumerate() {
                    let cid = first + offset as u32;
                    widths.push((cid, cid, width.as_number().unwrap_or(0.0) / 1000.0));
                }
                index += 2;
            }
            Object::Int(last) => {
                let width = items.get(index + 2).and_then(Object::as_number).unwrap_or(0.0);
                widths.push((first, last.max(0) as u32, width / 1000.0));
                index += 3;
            }
            _ => index += 1,
        }
    }
    Ok(widths)
}
