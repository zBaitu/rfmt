#![crate_name = "syntax"] // abcd
#![unstable(feature = "rustc_private", issue = "27812")] // aaaaaaaa
#![crate_type = "rlib"] // bbbbbbbbbb
#![crate_type = "dylib"]// cccccccccc

// gggggggggggggggggg
/* 
 * abcdefg
 * hijklmn
 */
#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://doc.rust-lang.org/favicon.ico", html_root_url = "https://doc.rust-lang.org/nightly/", test(attr(deny(warnings))))]
// ddddddddddddddd
#![cfg_attr(not(stage0),
deny(warnings))]

// aaaaaaaaaaaaa
const a: bool = true; // bbbbbbbbbbbbbbbbb
                        // cccccccccccccccc
                        // ddddddddddddddddd

const b: &'static str = br#"aaaaaaaaaaa"#;  // eeeeeeeeeee
pub const c: i32 = -12_345;// fffffffffffffffffff

// aaaaaaaaaaaaa
pub struct Point {
    // ddddddddddd
    pub x: i32, // aaa
    y: i32,// bbb
    pub x: i32, // ccc
}
