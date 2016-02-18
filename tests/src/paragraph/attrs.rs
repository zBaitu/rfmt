// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The Rust parser and macro expander.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![crate_name = "syntax"]
#![unstable(feature = "rustc_private", issue = "27812")]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
/* abcdefg
 * hijklmn
 */
#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://doc.rust-lang.org/favicon.ico", html_root_url = "https://doc.rust-lang.org/nightly/", test(attr(deny(warnings))))]
#![cfg_attr(not(stage0),
deny(warnings))]

#![feature(filling_drop)]
#![feature(rustc_private)]
#![feature(staged_api)]
#![feature(str_escape)]
#![feature(associated_consts)]
#![feature(unicode)]
#![feature(str_char)]
#![feature(libc)]

