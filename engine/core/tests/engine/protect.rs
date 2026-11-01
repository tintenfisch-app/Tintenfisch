use crate::support;

use std::fs;
use std::path::PathBuf;

use support::fixtures;
use support::rendering::{TestRenderer, library};
use support::synthetic_pdf::{Row, SyntheticPdf};
use tintenfisch::annotations::{Markup, MarkupStyle};
use tintenfisch::crypt::{EditRight, Permissions, Protection};
use tintenfisch::document::{Document, FieldInput};
use tintenfisch::error::ErrorKind;
use tintenfisch::file::SourceFile;
use tintenfisch::syntax::Object;

const NONE: Permissions = Permissions {
    print: false,
    print_faithful: false,
    modify: false,
    copy: false,
    annotate: false,
    fill_forms: false,
    assemble: false,
};

fn entropy(seed: u8) -> Vec<u8> {
    (0..Protection::ENTROPY).map(|index| (index as u8).wrapping_mul(31).wrapping_add(seed)).collect()
}

fn protection(user: &str, owner: &str, permissions: Permissions) -> Protection {
    Protection { user_password: user.into(), owner_password: owner.into(), permissions }
}

fn encrypted_fixture(file: &str) -> Vec<u8> {
    fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/encrypted/{file}.pdf"))).unwrap()
}

fn rendering(bytes: Vec<u8>, password: &str) -> Vec<u8> {
    TestRenderer::with_password(bytes, password).render(0, 1.0, None).rgba
}

#[test]
fn a_protected_file_opens_only_with_its_passwords_and_keeps_its_content() {
    let original = fixtures::editing_features();
    let mut document = Document::open(original.clone(), "").unwrap();
    let permissions = Permissions { print: true, copy: true, ..NONE };
    document.protect(&protection("open sesame", "boss", permissions), &entropy(1)).unwrap();
    assert!(document.protection().encrypted);
    let saved = document.save(false).unwrap();
    assert_eq!(Document::open(saved.clone(), "").err().unwrap().kind, ErrorKind::NeedsPassword);
    assert_eq!(Document::open(saved.clone(), "wrong").err().unwrap().kind, ErrorKind::NeedsPassword);
    let as_user = Document::open(saved.clone(), "open sesame").unwrap();
    assert_eq!(
        as_user.protection(),
        tintenfisch::document::ProtectionState { encrypted: true, permissions, owner_rights: false }
    );
    assert!(Document::open(saved.clone(), "boss").unwrap().protection().owner_rights);
    let file = SourceFile::open_with_password(saved.clone(), "boss").unwrap();
    let encrypt = file.trailer().get("Encrypt").and_then(Object::as_reference).unwrap();
    let dictionary = file.object(encrypt).unwrap();
    let dictionary = dictionary.as_dict().unwrap();
    assert_eq!((dictionary.int("V"), dictionary.int("R")), (Some(5), Some(6)));
    assert!(!saved.windows(b"Plain line".len()).any(|window| window == b"Plain line"), "the text is encrypted");
    assert_eq!(rendering(saved, "open sesame"), rendering(original, ""));
}

#[test]
fn a_permissions_password_alone_opens_freely_and_guards_the_protection() {
    let original = fixtures::editing_features();
    let mut document = Document::open(original.clone(), "").unwrap();
    document.protect(&protection("", "boss", NONE), &entropy(2)).unwrap();
    let saved = document.save(false).unwrap();
    let mut reopened = Document::open(saved, "").unwrap();
    assert_eq!(reopened.protection().permissions, NONE);
    assert!(!reopened.protection().owner_rights);
    assert_eq!(reopened.unprotect().err().unwrap().kind, ErrorKind::NeedsPassword);
    assert!(!reopened.grant_owner_rights("wrong"));
    assert!(reopened.grant_owner_rights("boss"));
    reopened.unprotect().unwrap();
    let plain = reopened.save(false).unwrap();
    assert!(!SourceFile::open(plain.clone()).unwrap().trailer().contains("Encrypt"));
    assert_eq!(rendering(plain, ""), rendering(original, ""));
}

#[test]
fn a_password_to_open_alone_also_lets_its_holder_change_the_protection() {
    let mut document = Document::open(fixtures::editing_features(), "").unwrap();
    document.protect(&protection("user", "", Permissions::ALL), &entropy(3)).unwrap();
    let saved = document.save(false).unwrap();
    assert!(Document::open(saved.clone(), "").is_err());
    let mut reopened = Document::open(saved, "user").unwrap();
    assert!(reopened.protection().owner_rights);
    let before = reopened.revision();
    reopened.unprotect().unwrap();
    assert_ne!(reopened.revision(), before, "changing the protection is a change to save");
    assert!(!SourceFile::open(reopened.save(false).unwrap()).unwrap().trailer().contains("Encrypt"));
    let mut document = Document::open(fixtures::editing_features(), "").unwrap();
    assert!(document.protect(&protection("", "", Permissions::ALL), &entropy(4)).is_err());
    assert!(document.protect(&protection("user", "boss", Permissions::ALL), &[0; 12]).is_err());
}

#[test]
fn a_file_another_program_protected_gets_new_passwords_once_the_owner_password_is_known() {
    let mut document = Document::open(encrypted_fixture("aes-256"), "secret").unwrap();
    let before = rendering(encrypted_fixture("aes-256"), "secret");
    assert!(!document.protection().owner_rights);
    assert!(document.protect(&protection("new", "boss", Permissions::ALL), &entropy(5)).is_err());
    assert!(document.grant_owner_rights("owner"));
    document.protect(&protection("new", "boss", Permissions::ALL), &entropy(5)).unwrap();
    let saved = document.save(false).unwrap();
    assert!(Document::open(saved.clone(), "secret").is_err());
    assert_eq!(rendering(saved, "new"), before);
}

#[test]
fn removing_the_protection_of_an_rc4_file_gives_a_plain_file() {
    let source = encrypted_fixture("rc4-128");
    let mut document = Document::open(source.clone(), "owner").unwrap();
    assert!(document.protection().owner_rights);
    document.unprotect().unwrap();
    assert!(!document.protection().encrypted);
    let plain = document.save(false).unwrap();
    assert!(!SourceFile::open(plain.clone()).unwrap().trailer().contains("Encrypt"));
    assert_eq!(rendering(plain, ""), rendering(source, "secret"));
}

fn versioned(header: &str, catalog_version: &str) -> Vec<u8> {
    let mut pdf = SyntheticPdf::new(header);
    pdf.object(1, 0, &format!("<< /Type /Catalog /Pages 2 0 R {catalog_version} >>"))
        .object(2, 0, fixtures::PAGES)
        .object(3, 0, fixtures::PAGE)
        .stream(4, "", fixtures::CONTENT);
    let rows: Vec<(u32, Row)> =
        std::iter::once((0, Row::Free(65535))).chain((1..=4).map(|num| (num, Row::InUse(num)))).collect();
    let start = pdf.xref_table(&rows, "/Size 5 /Root 1 0 R");
    pdf.startxref(start).finish()
}

fn protected_version(bytes: Vec<u8>) -> (Vec<u8>, Option<String>) {
    let mut document = Document::open(bytes, "").unwrap();
    document.protect(&protection("user", "boss", Permissions::ALL), &entropy(6)).unwrap();
    let saved = document.save(false).unwrap();
    let file = SourceFile::open_with_password(saved.clone(), "boss").unwrap();
    let catalog = file.object(file.trailer().get("Root").and_then(Object::as_reference).unwrap()).unwrap();
    let version =
        catalog.as_dict().unwrap().name("Version").map(|name| String::from_utf8_lossy(name.as_bytes()).into_owned());
    (saved[..9].to_vec(), version)
}

#[test]
fn aes_256_protection_declares_pdf_2_0_without_lowering_a_newer_version() {
    assert_eq!(protected_version(versioned("1.4", "")), (b"%PDF-2.0\n".to_vec(), None));
    assert_eq!(protected_version(versioned("1.4", "/Version /1.7")), (b"%PDF-2.0\n".to_vec(), Some("2.0".into())));
    assert_eq!(protected_version(versioned("2.0", "/Version /2.1")), (b"%PDF-2.0\n".to_vec(), Some("2.1".into())));
    assert_eq!(protected_version(versioned("2.1", "")), (b"%PDF-2.1\n".to_vec(), None));
}

fn restricted_form(user: &str, permissions: Permissions) -> Vec<u8> {
    let mut document = Document::open(fixtures::fillable_form(), "").unwrap();
    document.protect(&protection(user, "boss", permissions), &entropy(9)).unwrap();
    document.save(false).unwrap()
}

fn fill_last_name(document: &mut Document) -> tintenfisch::error::Result<()> {
    let field = document.fields().unwrap().into_iter().find(|field| field.name == "person.last").unwrap();
    document.set_field(field.id, FieldInput::Text("Lovelace".to_string()), &library()).map(|_| ())
}

#[test]
fn a_restricted_document_refuses_every_edit_its_permissions_withhold_until_the_owner_password_is_given() {
    let saved = restricted_form("", Permissions { print: true, ..NONE });
    let mut document = Document::open(saved.clone(), "").unwrap();
    assert_eq!(document.protection().withheld(), EditRight::ALL);
    let page = document.view(&library()).pages().unwrap()[0].id;
    let style = MarkupStyle { color: [1.0, 0.0, 0.0], width: 1.0, author: String::new(), date: String::new() };
    let note = Markup::Note { at: (20.0, 20.0), text: "note".to_string() };
    let field = document.fields().unwrap()[0].id;
    let refusals = [
        fill_last_name(&mut document).err(),
        document.add_markup(page, &note, &style, &library()).err(),
        document.delete_field(field).err(),
        document.rotate_pages(&[page], 1).err(),
        document.add_bookmark(None, 0, "Start", None).err(),
        document.apply_redactions(&library()).err(),
    ];
    for refusal in refusals {
        assert_eq!(refusal.map(|error| error.kind), Some(ErrorKind::NeedsPassword));
    }
    assert_eq!(document.save(false).unwrap(), saved, "an untouched restricted file still saves as it was");
    assert!(document.grant_owner_rights("boss"));
    assert!(document.protection().withheld().is_empty());
    fill_last_name(&mut document).unwrap();
    document.rotate_pages(&[page], 1).unwrap();
}

#[test]
fn a_document_that_allows_only_form_filling_takes_values_and_refuses_the_rest() {
    let saved = restricted_form("", Permissions { print: true, fill_forms: true, ..NONE });
    let mut document = Document::open(saved, "").unwrap();
    assert_eq!(
        document.protection().withheld(),
        [EditRight::Annotate, EditRight::DesignForms, EditRight::ModifyContent, EditRight::Assemble]
    );
    fill_last_name(&mut document).unwrap();
    let page = document.view(&library()).pages().unwrap()[0].id;
    assert_eq!(document.rotate_pages(&[page], 1).err().unwrap().kind, ErrorKind::NeedsPassword);
    let reopened = Document::open(document.save(false).unwrap(), "").unwrap();
    let last = reopened.fields().unwrap().into_iter().find(|field| field.name == "person.last").unwrap();
    assert_eq!(last.value, tintenfisch::forms::FieldValue::Text("Lovelace".to_string()));
}

#[test]
fn a_document_that_allows_comments_takes_notes_but_no_redaction_marks() {
    let saved = restricted_form("", Permissions { print: true, annotate: true, ..NONE });
    let mut document = Document::open(saved, "").unwrap();
    let page = document.view(&library()).pages().unwrap()[0].id;
    let style = MarkupStyle { color: [1.0, 0.0, 0.0], width: 1.0, author: String::new(), date: String::new() };
    let note = Markup::Note { at: (20.0, 20.0), text: "note".to_string() };
    document.add_markup(page, &note, &style, &library()).unwrap();
    let mark = Markup::RedactArea { from: (10.0, 10.0), to: (60.0, 40.0) };
    assert_eq!(document.add_markup(page, &mark, &style, &library()).err().unwrap().kind, ErrorKind::NeedsPassword);
    fill_last_name(&mut document).unwrap();
}

#[test]
fn opening_with_the_password_to_open_alone_keeps_the_restrictions_of_a_separate_owner_password() {
    let saved = restricted_form("open sesame", Permissions { print: true, ..NONE });
    let mut document = Document::open(saved.clone(), "open sesame").unwrap();
    assert_eq!(fill_last_name(&mut document).err().unwrap().kind, ErrorKind::NeedsPassword);
    let mut owner = Document::open(saved, "boss").unwrap();
    assert!(owner.protection().withheld().is_empty());
    fill_last_name(&mut owner).unwrap();
}

#[test]
#[ignore = "writes protected files to PREVIEW_DIR for checking them with other viewers"]
fn write_protected_preview() {
    let directory = std::env::var("PREVIEW_DIR").unwrap();
    let mut document = Document::open(fixtures::editing_features(), "").unwrap();
    document
        .protect(&protection("open sesame", "boss", Permissions { print: true, copy: true, ..NONE }), &entropy(7))
        .unwrap();
    fs::write(format!("{directory}/protected-user.pdf"), document.save(false).unwrap()).unwrap();
    let mut document = Document::open(fixtures::editing_features(), "").unwrap();
    document.protect(&protection("", "boss", Permissions { print: true, ..NONE }), &entropy(8)).unwrap();
    fs::write(format!("{directory}/protected-owner.pdf"), document.save(false).unwrap()).unwrap();
}
