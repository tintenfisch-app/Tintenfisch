use std::collections::HashMap;
use std::rc::Rc;

use crate::fonts::FallbackFace;

#[derive(Default)]
pub struct FontLibrary {
    faces: HashMap<FallbackFace, Rc<[u8]>>,
    replacements: HashMap<String, Vec<Rc<[u8]>>>,
    cjk: Option<Rc<[u8]>>,
}

impl FontLibrary {
    pub const CJK_FILE_NAME: &str = "NotoSansCJK-Subset.otf";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, face: FallbackFace, data: impl Into<Rc<[u8]>>) {
        self.faces.insert(face, data.into());
    }

    pub fn register_replacement(&mut self, key: &str, data: impl Into<Rc<[u8]>>) {
        self.replacements.entry(key.to_string()).or_default().push(data.into());
    }

    pub fn register_cjk(&mut self, data: impl Into<Rc<[u8]>>) {
        self.cjk = Some(data.into());
    }

    pub fn cjk_face(&self) -> Option<Rc<[u8]>> {
        self.cjk.clone()
    }

    pub fn replacements(&self, key: &str) -> &[Rc<[u8]>] {
        self.replacements.get(key).map_or(&[], Vec::as_slice)
    }

    pub fn face(&self, wanted: FallbackFace) -> Option<Rc<[u8]>> {
        let regular = FallbackFace { bold: false, italic: false, ..wanted };
        self.faces.get(&wanted).or_else(|| self.faces.get(&regular)).or_else(|| self.faces.values().next()).cloned()
    }
}
