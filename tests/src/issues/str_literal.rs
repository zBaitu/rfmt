const HEADER_STRUCT: &'static str = r#"
#[derive(Debug)]
pub struct Header {
    name: &'static [u8; 4],
    major: u8,
    minor: u8,
    revision: u8,
}
"#;

const HEADER_STRUCT: &'static str = br#"
#[derive(Debug)]
pub struct Header {
    name: &'static [u8; 4],
    major: u8,
    minor: u8,
    revision: u8,
}
"#;
