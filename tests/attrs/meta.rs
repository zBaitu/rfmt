#![macro_use(foo, bar)]
#![allow(unused, clippy::inline_always)]
#![link(name = "CoreFoundation", kind = "framework")]
#![doc = "example"]
#![clippy::inline_always]
#![no_std]

#![cfg(all(unix,
target_pointer_width = "32",
/* aaa */ b = "a"))]
