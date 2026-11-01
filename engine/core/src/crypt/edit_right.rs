use crate::crypt::Permissions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditRight {
    FillForms,
    Annotate,
    DesignForms,
    ModifyContent,
    Assemble,
}

impl EditRight {
    pub const ALL: [EditRight; 5] = [
        EditRight::FillForms,
        EditRight::Annotate,
        EditRight::DesignForms,
        EditRight::ModifyContent,
        EditRight::Assemble,
    ];

    pub fn granted_by(self, permissions: Permissions) -> bool {
        match self {
            EditRight::FillForms => permissions.fill_forms || permissions.annotate,
            EditRight::Annotate => permissions.annotate,
            EditRight::DesignForms => permissions.annotate && permissions.modify,
            EditRight::ModifyContent => permissions.modify,
            EditRight::Assemble => permissions.assemble,
        }
    }

    pub fn action(self) -> &'static str {
        match self {
            EditRight::FillForms => "filling in forms",
            EditRight::Annotate => "adding or changing comments",
            EditRight::DesignForms => "changing form fields",
            EditRight::ModifyContent => "editing the content",
            EditRight::Assemble => "organizing pages and bookmarks",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_filling_follows_either_flag_and_form_design_needs_both() {
        let none = Permissions::from_bits(0);
        assert!(EditRight::ALL.iter().all(|right| !right.granted_by(none)));
        assert!(EditRight::ALL.iter().all(|right| right.granted_by(Permissions::ALL)));
        assert!(EditRight::FillForms.granted_by(Permissions { annotate: true, ..none }));
        assert!(EditRight::FillForms.granted_by(Permissions { fill_forms: true, ..none }));
        assert!(!EditRight::DesignForms.granted_by(Permissions { annotate: true, ..none }));
        assert!(EditRight::DesignForms.granted_by(Permissions { annotate: true, modify: true, ..none }));
        assert!(!EditRight::Assemble.granted_by(Permissions { modify: true, ..none }));
    }
}
