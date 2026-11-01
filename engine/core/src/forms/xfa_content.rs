use crate::error::Result;
use crate::forms::FieldTree;
use crate::store::ObjectStore;
use crate::syntax::Object;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XfaContent {
    Absent,
    Hybrid,
    Dynamic,
}

impl XfaContent {
    pub fn of(store: &ObjectStore) -> Result<XfaContent> {
        let Some(form) = FieldTree::new(store).form()? else { return Ok(XfaContent::Absent) };
        if form.get("XFA").is_none_or(|xfa| matches!(xfa, Object::Null)) {
            return Ok(XfaContent::Absent);
        }
        let catalog = match store.trailer().get("Root") {
            Some(root) => store.resolve(root)?,
            None => Object::Null,
        };
        let needs_rendering = match catalog.as_dict().and_then(|catalog| catalog.get("NeedsRendering")) {
            Some(flag) => store.resolve(flag)?.as_bool().unwrap_or(false),
            None => false,
        };
        Ok(if needs_rendering { XfaContent::Dynamic } else { XfaContent::Hybrid })
    }
}
