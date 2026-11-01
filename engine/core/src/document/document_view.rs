use std::rc::Rc;

use crate::document::{DocumentCaches, PageFont, PageSummary};
use crate::error::{Error, Result};
use crate::fonts::{CidFont, FontLibrary, FontProgram};
use crate::pages::{Page, PageId, PageTree, drawn_forms};
use crate::render::{PageRenderer, Raster, RenderContext, Tile};
use crate::store::ObjectStore;
use crate::syntax::{Dict, Object};
use crate::text::{TextExtractor, TextGlyph, TextLayout};

#[derive(Clone, Copy)]
pub struct DocumentView<'a> {
    pub store: &'a ObjectStore,
    pub caches: &'a DocumentCaches,
    pub library: &'a FontLibrary,
}

impl<'a> DocumentView<'a> {
    pub fn pages(&self) -> Result<Rc<Vec<Page>>> {
        let revision = self.store.revision();
        if let Some((cached, pages)) = &*self.caches.pages.borrow()
            && *cached == revision
        {
            return Ok(pages.clone());
        }
        let pages = Rc::new(PageTree::new(self.store).pages()?);
        *self.caches.pages.borrow_mut() = Some((revision, pages.clone()));
        Ok(pages)
    }

    pub fn page(&self, id: PageId) -> Result<Page> {
        self.pages()?.iter().find(|page| page.id == id).cloned().ok_or_else(|| Error::new(format!("no page {id}")))
    }

    pub fn summaries(&self) -> Result<Vec<PageSummary>> {
        Ok(self
            .pages()?
            .iter()
            .map(|page| {
                let (width, height) = page.display_size();
                PageSummary { id: page.id, width, height, rotate: page.rotate }
            })
            .collect())
    }

    pub fn render(&self, id: PageId, scale: f64, tile: Option<Tile>) -> Result<Raster> {
        let page = self.page(id)?;
        let context = RenderContext {
            store: self.store,
            fonts: &self.caches.fonts,
            images: &self.caches.images,
            library: self.library,
        };
        PageRenderer::new(context).render(&page, scale, tile)
    }

    pub fn text_glyphs(&self, id: PageId) -> Result<Vec<TextGlyph>> {
        TextExtractor::new(self.store, &self.caches.fonts, self.library).page(&self.page(id)?)
    }

    pub fn fonts(&self, id: PageId) -> Result<Vec<PageFont>> {
        let page = self.page(id)?;
        let Some(fonts) = page.resources.get("Font") else { return Ok(Vec::new()) };
        let fonts = self.store.resolve(fonts)?.as_dict().cloned().unwrap_or_default();
        Ok(fonts
            .iter()
            .filter_map(|(name, entry)| {
                let font = self.caches.fonts.font(entry, self.store, self.library).ok()?;
                Some(PageFont {
                    name: name.clone(),
                    family: font.family_name().map(String::from),
                    base_font: font.base_font().to_string(),
                    embedded: font.embedded_program().is_some(),
                    face: font.fallback_face(),
                    weight: font.embedded_program().and_then(FontProgram::weight),
                })
            })
            .collect())
    }

    pub fn needs_cjk_substitute(&self, id: PageId) -> Result<bool> {
        let page = self.page(id)?;
        let mut resources = vec![page.resources.clone()];
        for form in drawn_forms(self.store, &page.resources) {
            let object = self.store.object(form)?;
            if let Some(own) = object.as_stream().and_then(|stream| stream.dict.get("Resources")) {
                resources.push(self.store.resolve(own)?.as_dict().cloned().unwrap_or_default());
            }
        }
        Ok(resources.iter().any(|resources| self.uses_cjk_substitute(resources)))
    }

    fn uses_cjk_substitute(&self, resources: &Dict) -> bool {
        let Some(fonts) = resources.get("Font").and_then(|fonts| self.store.resolve(fonts).ok()) else {
            return false;
        };
        fonts.as_dict().is_some_and(|fonts| {
            fonts.iter().any(|(_, entry)| {
                self.store
                    .resolve(entry)
                    .ok()
                    .as_ref()
                    .and_then(Object::as_dict)
                    .is_some_and(|font| CidFont::needs_cjk_substitute(font, self.store))
            })
        })
    }

    pub fn text_layout(&self, id: PageId) -> Result<Rc<TextLayout>> {
        let key = (id, self.store.revision());
        if let Some(layout) = self.caches.layouts.borrow().get(&key) {
            return Ok(layout.clone());
        }
        let layout = Rc::new(TextLayout::new(&self.text_glyphs(id)?));
        self.caches.layouts.borrow_mut().insert(key, layout.clone());
        Ok(layout)
    }
}
