use std::collections::HashSet;

use crate::error::{Error, Result};
use crate::geometry::Rect;
use crate::pages::page::normalized_rotation;
use crate::pages::{Page, PageId};
use crate::store::ObjectStore;
use crate::syntax::{Dict, ObjId, Object};

const MAX_DEPTH: usize = 64;

#[derive(Clone, Default)]
struct Inherited {
    resources: Option<Object>,
    media_box: Option<Rect>,
    crop_box: Option<Rect>,
    rotate: Option<i64>,
}

impl Inherited {
    fn below(&self, node: &Dict) -> Inherited {
        Inherited {
            resources: node.get("Resources").cloned().or_else(|| self.resources.clone()),
            media_box: node.get("MediaBox").and_then(Rect::from_object).or(self.media_box),
            crop_box: node.get("CropBox").and_then(Rect::from_object).or(self.crop_box),
            rotate: node.get("Rotate").and_then(Object::as_int).or(self.rotate),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Placement {
    id: ObjId,
    parent: Option<ObjId>,
    index: usize,
}

struct Mended {
    id: ObjId,
    parent: Option<ObjId>,
    kids: Option<(Vec<Object>, i64)>,
}

pub struct PageTree<'a> {
    store: &'a ObjectStore,
}

impl<'a> PageTree<'a> {
    pub fn new(store: &'a ObjectStore) -> Self {
        Self { store }
    }

    pub fn pages(&self) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        self.walk(&mut |placement, inherited, dict| {
            pages.push(self.page(placement, inherited, dict)?);
            Ok(())
        })?;
        Ok(pages)
    }

    pub fn separate_shared_pages(store: &mut ObjectStore) -> Result<usize> {
        let mut seen = HashSet::new();
        let mut shared = Vec::new();
        PageTree::new(store).walk(&mut |placement, _, _| {
            if !seen.insert(placement.id) {
                shared.push(placement);
            }
            Ok(())
        })?;
        if shared.is_empty() {
            return Ok(0);
        }
        store.repair(|transaction| {
            for placement in &shared {
                let Some(parent) = placement.parent else { continue };
                let original = transaction.object(placement.id)?;
                let copy = transaction.allocate((*original).clone());
                let mut node = transaction.object(parent)?.as_dict().cloned().unwrap_or_default();
                let mut kids = node.get("Kids").and_then(Object::as_array).map(<[Object]>::to_vec).unwrap_or_default();
                if let Some(slot) = kids.get_mut(placement.index) {
                    *slot = Object::Ref(copy);
                }
                node.insert("Kids", Object::Array(kids));
                transaction.put(parent, Object::Dict(node))?;
            }
            Ok(shared.len())
        })
    }

    pub fn break_cycles(store: &mut ObjectStore) -> Result<usize> {
        let tree = PageTree::new(store);
        let (mut visited, mut mended, mut cut) = (HashSet::new(), Vec::new(), 0);
        tree.mend(tree.root()?, None, 0, &mut visited, &mut mended, &mut cut)?;
        if cut == 0 {
            return Ok(0);
        }
        store.repair(|transaction| {
            for node in &mended {
                let mut dict = transaction.object(node.id)?.as_dict().cloned().unwrap_or_default();
                match node.parent {
                    Some(parent) => dict.insert("Parent", Object::Ref(parent)),
                    None => {
                        dict.remove("Parent");
                    }
                }
                if let Some((kids, count)) = &node.kids {
                    dict.insert("Kids", Object::Array(kids.clone()));
                    dict.insert("Count", Object::Int(*count));
                }
                transaction.put(node.id, Object::Dict(dict))?;
            }
            Ok(cut)
        })
    }

    fn mend(
        &self,
        id: ObjId,
        parent: Option<ObjId>,
        depth: usize,
        visited: &mut HashSet<ObjId>,
        mended: &mut Vec<Mended>,
        cut: &mut usize,
    ) -> Result<i64> {
        if depth > MAX_DEPTH {
            return Err(Error::new("page tree nested too deeply"));
        }
        let object = self.store.object(id)?;
        let Some(node) = object.as_dict() else {
            return Ok(0);
        };
        let kids = match node.get("Kids") {
            Some(kids) => self.store.resolve(kids)?,
            None => Object::Null,
        };
        if !is_tree_node(node, &kids) {
            mended.push(Mended { id, parent, kids: None });
            return Ok(1);
        }
        visited.insert(id);
        let (mut kept, mut count) = (Vec::new(), 0);
        for kid in kids.as_array().unwrap_or_default() {
            match kid.as_reference() {
                Some(child) if visited.contains(&child) => *cut += 1,
                Some(child) => {
                    count += self.mend(child, Some(id), depth + 1, visited, mended, cut)?;
                    kept.push(kid.clone());
                }
                None => kept.push(kid.clone()),
            }
        }
        mended.push(Mended { id, parent, kids: Some((kept, count)) });
        Ok(count)
    }

    pub fn node_ids(&self) -> Result<HashSet<ObjId>> {
        let root = self.root()?;
        let mut nodes = HashSet::from([root]);
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            let object = self.store.object(node)?;
            let kids = object.as_dict().and_then(|dict| dict.get("Kids")).cloned().unwrap_or(Object::Null);
            for kid in self.store.resolve(&kids)?.as_array().unwrap_or_default() {
                let Some(id) = kid.as_reference() else { continue };
                let child = self.store.object(id)?;
                if child.as_dict().is_some_and(|dict| dict.has_type("Pages")) && nodes.insert(id) {
                    pending.push(id);
                }
            }
        }
        Ok(nodes)
    }

    fn root(&self) -> Result<ObjId> {
        let catalog = self.store.object(self.catalog_id()?)?;
        catalog
            .as_dict()
            .and_then(|catalog| catalog.get("Pages"))
            .and_then(Object::as_reference)
            .ok_or_else(|| Error::new("the catalog has no page tree"))
    }

    fn catalog_id(&self) -> Result<ObjId> {
        self.store
            .trailer()
            .get("Root")
            .and_then(Object::as_reference)
            .ok_or_else(|| Error::new("trailer has no /Root"))
    }

    fn walk(&self, visit: &mut dyn FnMut(Placement, &Inherited, &Dict) -> Result<()>) -> Result<()> {
        let mut visited = HashSet::new();
        let root = Placement { id: self.root()?, parent: None, index: 0 };
        self.walk_node(root, &Inherited::default(), 0, &mut visited, visit)
    }

    fn walk_node(
        &self,
        placement: Placement,
        inherited: &Inherited,
        depth: usize,
        visited: &mut HashSet<ObjId>,
        visit: &mut dyn FnMut(Placement, &Inherited, &Dict) -> Result<()>,
    ) -> Result<()> {
        if depth > MAX_DEPTH {
            return Err(Error::new("page tree nested too deeply"));
        }
        let object = self.store.object(placement.id)?;
        let Some(node) = object.as_dict() else {
            return Ok(());
        };
        let kids = match node.get("Kids") {
            Some(kids) => self.store.resolve(kids)?,
            None => Object::Null,
        };
        if !is_tree_node(node, &kids) {
            return visit(placement, &inherited.below(node), node);
        }
        if !visited.insert(placement.id) {
            return Ok(());
        }
        let below = inherited.below(node);
        for (index, kid) in kids.as_array().unwrap_or_default().iter().enumerate() {
            if let Some(id) = kid.as_reference() {
                let child = Placement { id, parent: Some(placement.id), index };
                self.walk_node(child, &below, depth + 1, visited, visit)?;
            }
        }
        Ok(())
    }

    fn page(&self, placement: Placement, inherited: &Inherited, dict: &Dict) -> Result<Page> {
        let resources = match &inherited.resources {
            Some(resources) => self.store.resolve(resources)?.as_dict().cloned().unwrap_or_default(),
            None => Dict::new(),
        };
        Ok(Page {
            id: PageId(placement.id),
            parent: placement.parent,
            media_box: inherited.media_box.unwrap_or_else(Page::default_media_box),
            crop_box: inherited.crop_box,
            rotate: normalized_rotation(inherited.rotate.unwrap_or(0)),
            resources,
            contents: self.contents(dict)?,
        })
    }

    fn contents(&self, page: &Dict) -> Result<Vec<ObjId>> {
        let Some(contents) = page.get("Contents") else {
            return Ok(Vec::new());
        };
        Ok(match contents {
            Object::Ref(id) => match &*self.store.object(*id)? {
                Object::Array(items) => items.iter().filter_map(Object::as_reference).collect(),
                _ => vec![*id],
            },
            Object::Array(items) => items.iter().filter_map(Object::as_reference).collect(),
            _ => Vec::new(),
        })
    }
}

fn is_tree_node(node: &Dict, kids: &Object) -> bool {
    node.has_type("Pages") || (!node.has_type("Page") && kids.as_array().is_some())
}
