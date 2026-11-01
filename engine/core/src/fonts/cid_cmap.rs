use crate::syntax::Name;
use crate::syntax::lexer::{Lexer, Token};

const FIRST_ASCII_CID: u32 = 1;

#[derive(Clone, Debug, PartialEq)]
struct CodespaceRange {
    length: usize,
    low: u32,
    high: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct CidRange {
    length: usize,
    low: u32,
    high: u32,
    cid: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum UnicodeForm {
    Ucs2,
    Utf16,
    Utf8,
    Utf32,
}

impl UnicodeForm {
    fn of(name: &[u8]) -> Option<UnicodeForm> {
        let mut parts = name.split(|&byte| byte == b'-');
        if !parts.next()?.starts_with(b"Uni") {
            return None;
        }
        match parts.next()? {
            b"UCS2" => Some(UnicodeForm::Ucs2),
            b"UTF16" => Some(UnicodeForm::Utf16),
            b"UTF8" => Some(UnicodeForm::Utf8),
            b"UTF32" => Some(UnicodeForm::Utf32),
            _ => None,
        }
    }

    fn codespaces(self) -> Vec<CodespaceRange> {
        let range = |length, low, high| CodespaceRange { length, low, high };
        match self {
            UnicodeForm::Ucs2 => vec![range(2, 0, 0xFFFF)],
            UnicodeForm::Utf16 => vec![range(2, 0, 0xD7FF), range(2, 0xE000, 0xFFFF), range(4, 0xD800DC00, 0xDBFFDFFF)],
            UnicodeForm::Utf8 => vec![
                range(1, 0, 0x7F),
                range(2, 0xC280, 0xDFBF),
                range(3, 0xE0A080, 0xEFBFBF),
                range(4, 0xF0908080, 0xF48FBFBF),
            ],
            UnicodeForm::Utf32 => vec![range(4, 0, 0x10FFFF)],
        }
    }

    fn decode(self, code: u32) -> Option<char> {
        match self {
            UnicodeForm::Ucs2 | UnicodeForm::Utf32 => char::from_u32(code),
            UnicodeForm::Utf16 if code > 0xFFFF => char::decode_utf16([(code >> 16) as u16, code as u16]).next()?.ok(),
            UnicodeForm::Utf16 => char::from_u32(code),
            UnicodeForm::Utf8 => {
                let bytes = code.to_be_bytes();
                let first = bytes.iter().position(|&byte| byte != 0).unwrap_or(3);
                std::str::from_utf8(&bytes[first..]).ok()?.chars().next()
            }
        }
    }

    fn encode(self, character: char) -> Option<(u32, usize)> {
        match self {
            UnicodeForm::Ucs2 => u16::try_from(character as u32).ok().map(|unit| (unit as u32, 2)),
            UnicodeForm::Utf16 => {
                let mut units = [0; 2];
                let units = character.encode_utf16(&mut units);
                Some((units.iter().fold(0, |code, &unit| code << 16 | unit as u32), units.len() * 2))
            }
            UnicodeForm::Utf8 => {
                let mut bytes = [0; 4];
                let bytes = character.encode_utf8(&mut bytes);
                Some((number(bytes.as_bytes()), bytes.len()))
            }
            UnicodeForm::Utf32 => Some((character as u32, 4)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CidCmap {
    codespaces: Vec<CodespaceRange>,
    ranges: Vec<CidRange>,
    identity: bool,
    unicode: Option<UnicodeForm>,
    pub vertical: bool,
}

impl CidCmap {
    pub fn identity(vertical: bool) -> Self {
        Self {
            codespaces: vec![CodespaceRange { length: 2, low: 0, high: 0xFFFF }],
            ranges: Vec::new(),
            identity: true,
            unicode: None,
            vertical,
        }
    }

    pub fn is_identity(&self) -> bool {
        self.identity
    }

    pub fn is_unicode(&self) -> bool {
        self.unicode.is_some()
    }

    pub fn predefined(name: &Name) -> Self {
        let vertical = name.as_bytes().ends_with(b"-V");
        match UnicodeForm::of(name.as_bytes()) {
            Some(form) => Self {
                codespaces: form.codespaces(),
                ranges: Vec::new(),
                identity: false,
                unicode: Some(form),
                vertical,
            },
            None => Self::identity(vertical),
        }
    }

    pub fn unicode(&self, code: u32) -> Option<char> {
        self.unicode?.decode(code)
    }

    pub fn code_for(&self, character: char) -> Option<(u32, usize)> {
        self.unicode?.encode(character)
    }

    pub fn parse(data: &[u8]) -> Self {
        let mut cmap =
            CidCmap { codespaces: Vec::new(), ranges: Vec::new(), identity: false, unicode: None, vertical: false };
        let mut lexer = Lexer::new(data, 0);
        let mut operands: Vec<Token> = Vec::new();
        while let Ok(Some(token)) = lexer.next_token() {
            match token {
                Token::Keyword(b"endcodespacerange") => {
                    for pair in operands.chunks(2) {
                        if let [Token::String(low), Token::String(high)] = pair {
                            cmap.codespaces.push(CodespaceRange {
                                length: low.len(),
                                low: number(low),
                                high: number(high),
                            });
                        }
                    }
                    operands.clear();
                }
                Token::Keyword(b"endcidrange") => {
                    for triple in operands.chunks(3) {
                        if let [Token::String(low), Token::String(high), Token::Int(cid)] = triple {
                            cmap.ranges.push(CidRange {
                                length: low.len(),
                                low: number(low),
                                high: number(high),
                                cid: *cid as u32,
                            });
                        }
                    }
                    operands.clear();
                }
                Token::Keyword(b"endcidchar") => {
                    for pair in operands.chunks(2) {
                        if let [Token::String(code), Token::Int(cid)] = pair {
                            let value = number(code);
                            cmap.ranges.push(CidRange {
                                length: code.len(),
                                low: value,
                                high: value,
                                cid: *cid as u32,
                            });
                        }
                    }
                    operands.clear();
                }
                Token::Name(name) if name.is("WMode") => {
                    if let Ok(Some(Token::Int(mode))) = lexer.next_token() {
                        cmap.vertical = mode == 1;
                    }
                }
                Token::Keyword(word) if word.starts_with(b"begin") => operands.clear(),
                Token::Keyword(_) => operands.clear(),
                other => operands.push(other),
            }
        }
        if cmap.codespaces.is_empty() {
            cmap.codespaces.push(CodespaceRange { length: 2, low: 0, high: 0xFFFF });
        }
        cmap
    }

    pub fn next_code(&self, bytes: &[u8]) -> (u32, usize) {
        for length in 1..=4.min(bytes.len()) {
            let value = number(&bytes[..length]);
            if self.codespaces.iter().any(|range| range.length == length && (range.low..=range.high).contains(&value)) {
                return (value, length);
            }
        }
        let length = self.codespaces.iter().map(|range| range.length).min().unwrap_or(1).min(bytes.len()).max(1);
        (number(&bytes[..length]), length)
    }

    pub fn cid(&self, code: u32, length: usize) -> u32 {
        if self.identity {
            return code;
        }
        if self.unicode.is_some() {
            return match self.unicode(code) {
                Some(character @ ' '..='~') => character as u32 - ' ' as u32 + FIRST_ASCII_CID,
                _ => 0,
            };
        }
        self.ranges
            .iter()
            .rev()
            .find(|range| range.length == length && (range.low..=range.high).contains(&code))
            .map_or(0, |range| range.cid + (code - range.low))
    }
}

fn number(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0, |value, &byte| value << 8 | byte as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_mixed_width_codes_and_maps_ranges() {
        let cmap = CidCmap::parse(
            b"/WMode 0 def 2 begincodespacerange <00> <80> <8140> <FFFF> endcodespacerange
              1 begincidrange <20> <7e> 1 endcidrange
              1 begincidrange <8140> <817e> 633 endcidrange
              1 begincidchar <8150> 9000 endcidchar",
        );
        assert_eq!(cmap.next_code(b"A\x81\x41"), (0x41, 1));
        assert_eq!(cmap.next_code(b"\x81\x41"), (0x8141, 2));
        assert_eq!(cmap.cid(0x41, 1), 34);
        assert_eq!(cmap.cid(0x8141, 2), 634);
        assert_eq!(cmap.cid(0x8150, 2), 9000);
        assert!(!cmap.vertical);
    }

    #[test]
    fn treats_identity_as_two_byte_codes() {
        let cmap = CidCmap::identity(false);
        assert_eq!(cmap.next_code(b"\x01\x02\x03"), (0x0102, 2));
        assert_eq!(cmap.cid(0x0102, 2), 0x0102);
        assert!(CidCmap::predefined(&Name::from("Identity-V")).vertical);
    }

    #[test]
    fn reads_predefined_unicode_cmaps_as_their_own_code_points() {
        let ucs2 = CidCmap::predefined(&Name::from("UniGB-UCS2-H"));
        assert_eq!(
            ucs2.next_code("中A".encode_utf16().flat_map(u16::to_be_bytes).collect::<Vec<_>>().as_slice()),
            (0x4E2D, 2)
        );
        assert_eq!(ucs2.unicode(0x4E2D), Some('中'));
        assert_eq!(ucs2.cid(0x41, 2), 34);
        assert_eq!(ucs2.cid(0x4E2D, 2), 0);
        assert_eq!(ucs2.code_for('日'), Some((0x65E5, 2)));
        assert!(!ucs2.vertical);

        let utf16 = CidCmap::predefined(&Name::from("UniJIS-UTF16-V"));
        assert!(utf16.vertical);
        assert_eq!(utf16.next_code(b"\xD8\x40\xDC\x0B\x30\x42"), (0xD840DC0B, 4));
        assert_eq!(utf16.unicode(0xD840DC0B), Some('\u{2000B}'));
        assert_eq!(utf16.code_for('\u{2000B}'), Some((0xD840DC0B, 4)));
        assert_eq!(utf16.next_code(b"\x30\x42"), (0x3042, 2));

        let utf8 = CidCmap::predefined(&Name::from("UniKS-UTF8-H"));
        let bytes = "A한".as_bytes();
        assert_eq!(utf8.next_code(bytes), (0x41, 1));
        let (code, length) = utf8.next_code(&bytes[1..]);
        assert_eq!((utf8.unicode(code), length), (Some('한'), 3));
        assert_eq!(utf8.code_for('한'), Some((code, 3)));

        let utf32 = CidCmap::predefined(&Name::from("UniCNS-UTF32-H"));
        assert_eq!(utf32.next_code(&[0, 0, 0x4E, 0x2D]), (0x4E2D, 4));
        assert_eq!(utf32.unicode(0x4E2D), Some('中'));

        assert!(CidCmap::predefined(&Name::from("UniJIS-UCS2-HW-H")).is_unicode());
        assert!(!CidCmap::predefined(&Name::from("GBK-EUC-H")).is_unicode());
    }
}
