# rFmt ---- Rust source code formatter

https://github.com/zBaitu/rfmt


# Overview
rfmt is a Rust source code formatter. Yes, there is already an official tool [rustfmt](https://github.com/rust-lang-nursery/rustfmt) from [Rust Nursery](https://github.com/rust-lang-nursery). So why write another one?
* rustfmt is great for configurable, but there are still some style that i don't like in my personal taste.
* Write a code formatter for Rust can make me learn Rust more deeply, for example, the AST of Rust.
* For fun: )


# Version
Support for [Rust 1.36.0](https://blog.rust-lang.org/2019/07/04/Rust-1.36.0.html)


# Install, Build
* Install

```
cargo install rfmt
```
* Build
```
git clone git@github.com:zBaitu/rfmt.git
cargo build --release
```


# Usage
```
rfmt 1.36.0
baitu <zbaitu@gmail.com>
Another Rust source code formatter.

USAGE:
    rfmt [FLAGS] [input]

FLAGS:
    -a, --ast          Print the rust original syntax ast debug info
    -c, --check        Check exceed lines and trailing white space lines
    -d, --debug        Print the rfmt ir debug info
    -h, --help         Prints help information
    -o, --overwrite    Overwrite the source file
    -p, --print        Print the rfmt ir simple format
    -V, --version      Prints version information

ARGS:
    <input>    Input file or dir. If `input` is a dir, rfmt will do action for all files in this dir recursively. If
               neither `options` nor `input` is specified, rfmt will format source code from stdin.
```


# Running rfmt from your editor(Copy from rustfmt)
* [Vim](http://johannh.me/blog/rustfmt-vim.html)
* [Emacs](https://github.com/fbergroth/emacs-rustfmt)
* [Sublime Text 3](https://packagecontrol.io/packages/BeautifyRust)
* [Atom](atom.md)
* Visual Studio Code using [RustyCode](https://github.com/saviorisdead/RustyCode) or [vsc-rustfmt](https://github.com/Connorcpu/vsc-rustfmt)

In fact, I only use rfmt for Vim now. I do not test for other editors. It is just to replace `rustfmt` to `rfmt`. For example, Vim:
```
let g:formatdef_rfmt = '"rfmt"'
let g:formatters_rust = ['rfmt']
```


# Features
Comparing to **rustfmt**, there are some main different features from **rfmt**:
* Keep wrap from user input.
* Different align strategy.
* Group `crate`, `use`, `mod`, `attributes` and sort them.
* **DO NOT** format `doc`, `comment`, `string`. You can use the **check** function to show exceed lines and trailing white space lines.
* Provide check, directory recursively, ast dump, debug.
* Nightly features.

The following part will show such features in detail, with some existing issues from rustfmt.

### Keep wrap from user input
For the issue: [rustfmt reformats bit manipiulations](https://github.com/rust-lang-nursery/rustfmt/issues/626).
```
fn main() {
    let (a, b, c, d) = (0, 0, 0, 0);
    let _ = u32::from_be(((a as u32) << 24) |
                         ((b as u32) << 16) |
                         ((c as u32) <<  8) |
                          (d as u32) <<  0);
}
```
* rustfmt
```
fn main() {
    let (a, b, c, d) = (0, 0, 0, 0);
    let _ =
        u32::from_be(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32) << 0);
}
```
Of cause you can use `#[rustfmt_skip]` to avoid such code, but in my personal opinon, I really don't like to add other code just for the source formatting tool.
* rfmt
```
fn main() {
    let (a, b, c, d) = (0, 0, 0, 0);
    let _ = u32::from_be(((a as u32) << 24) | 
                         ((b as u32) << 16) | 
                         ((c as u32) << 8) | 
                         (d as u32) << 0);
}
```
It looks OK, isn't it? Why rfmt can keep the user wrap? Because of the [rfmt ir](https://github.com/zBaitu/rFmt/blob/master/src/ir.rs). The custom ir of Rust AST record location information of every element as far as possible. Look another example:
```
fn main() {
    let ref_packet = [0xde, 0xf0, 0x12, 0x34, 0x45, 0x67,
                     0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
                     0x86, 0xdd];
}
```
* rustfmt
```
fn main() {
    let ref_packet = [
        0xde, 0xf0, 0x12, 0x34, 0x45, 0x67, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0x86, 0xdd,
    ];
}
```
* rfmt
```
fn main() {
    let ref_packet = [0xde, 0xf0, 0x12, 0x34, 0x45, 0x67,
                      0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
                      0x86, 0xdd];
}
```

### Different align strategy
I prefer to put parameters on one line as much as possible. 
```
fn main() {
    f(123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```
* rustfmt
```
fn main() {
    f(
        123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg",
        "hijklmn", 0987654321, "opqrst", "uvwxyz",
    );
}
```
* rfmt
```
fn main() {
    f(123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg",
      "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```

If the left align position is beyond limit(It is **40** for now), rfmt prefer double indent align to function call align. rfmt make source code left lean, while rustfmt is right lean, I think. An exsiting issue: [rustfmt should avoid rightwards drifting big blocks of code](https://github.com/rust-lang-nursery/rustfmt/issues/439)
````
fn main() {
    let mut arms = variants.iter().enumerate().map(|(i, &(ident, v_span, ref summary))| {
        let i_expr = cx.expr_usize(v_span, i);
        let pat = cx.pat_lit(v_span, i_expr);

        let path = cx.path(v_span, vec![substr.type_ident, ident]);
        let thing = rand_thing(cx, v_span, path, summary, |cx, sp| rand_call(cx, sp));
        cx.arm(v_span, vec![ pat ], thing)
    }).collect::<Vec<ast::Arm> >();
}
````
* rustfmt
```
fn main() {
    let mut arms = variants
        .iter()
        .enumerate()
        .map(|(i, &(ident, v_span, ref summary))| {
            let i_expr = cx.expr_usize(v_span, i);
            let pat = cx.pat_lit(v_span, i_expr);

            let path = cx.path(v_span, vec![substr.type_ident, ident]);
            let thing = rand_thing(cx, v_span, path, summary, |cx, sp| rand_call(cx, sp));
            cx.arm(v_span, vec![pat], thing)
        })
        .collect::<Vec<ast::Arm>>();
}
```
* rfmt
```
fn main() {
    let mut arms = variants.iter().enumerate().map(|(i, &(ident, v_span, ref summary))| {
        let i_expr = cx.expr_usize(v_span, i);
        let pat = cx.pat_lit(v_span, i_expr);
        let path = cx.path(v_span, vec![substr.type_ident, ident]);
        let thing = rand_thing(cx, v_span, path, summary, |cx, sp| rand_call(cx, sp));
        cx.arm(v_span, vec![pat], thing)
    }).collect::<Vec<ast::Arm>>();
}
```
The result from rfmt is not changed, because this source code fits rfmt's code style.

### Group `crate`, `use`, `mod`, `attributes` and sort them
```
#![feature(custom_derive)]
#![deny(warnings)]
#![feature(question_mark)]
#![feature(iter_arith)]
#![feature(rustc_private)]

extern crate rst;
extern crate getopts;
extern crate walkdir;

use std::env;
use getopts::Options;

#[macro_use]
mod ts;

mod ir;
mod ft;
mod tr;
mod rfmt;
```
* rfmt
```
#![deny(warnings)]
#![feature(custom_derive)]
#![feature(iter_arith)]
#![feature(question_mark)]
#![feature(rustc_private)]

extern crate getopts;
extern crate rst;
extern crate walkdir;

use getopts::Options;
use std::env;

#[macro_use]
mod ts;

mod ft;
mod ir;
mod rfmt;
mod tr;
```
rfmt only group items that appear continuously. If on item is special that it must keep its order, like the `mod ts;`, make it separate from others.

### **DO NOT** format `doc`, `comment`, `string`
There are many issues about doc, comment, string, raw string from rustfmt. I think such element can leave free for user to write anything, any format they want. 

### Provide check, directory recursively, ast dump
If you want to check is there some line break the code style limit, rfmt provide check function.
```
// aaaaa  
// bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
fn main() {
    let a = r#"aaaaaaaaaaaaaaaaaaaaaaaaaaaa  
            bbbbbbbbbbbbbbbbb"#;
}
```
```
rfmt -c g.rs

"g.rs"
trailing_ws_lines: {1, 4}

------------------------------------------------------------------------------------------------------------------------
````

You can check or overwrite all files in a directory.
```
rfmt -c rust/src/libcore
rfmt -o rust/src/libstd
```

Maybe you are interested to see the Rust AST of a source code.
```
// AST
fn main() {}
```
```
rfmt -a a.rs
```
```
Crate {
    loc: Loc(7, 19, nl),
    attrs: [],
    module: Mod {
        loc: Loc(7, 19, nl),
        name: "main",
        items: [
            Item {
                loc: Loc(7, 19, nl),
                attrs: [],
                vis: "",
                item: Fn(
                    Fn {
                        header: FnHeader {
                            is_unsafe: false,
                            is_async: false,
                            is_const: false,
                            abi: "\"Rust\"",
                        },
                        name: "main",
                        generics: Generics {
                            lifetime_defs: [],
                            type_params: [],
                            wh: Where {
                                clauses: [],
                            },
                        },
                        sig: FnSig {
                            args: [],
                            ret: Return {
                                nl: false,
                                ret: None,
                            },
                        },
                        block: Block {
                            loc: Loc(17, 19),
                            is_unsafe: false,
                            stmts: [],
                        },
                    },
                ),
            },
        ],
    },
}

------------------------------------------------------------------------------------------------------------------------

{
    7: [
        "// AST",
    ],
}
{}
```


# Drawbacks
As rfmt is written as a personal tool(toy) for my daily develop, it lacks some common features now.
* No config  
rustfmt provide lots of config option, but rfmt provide none. Code style is something like food, everyone has his taste. Although rustfmt has much configs now, there are still new config require open in issues. If majority part of rfmt's style suit your taste, you can clone and make some small modification, such as **LF**, **max width**, **indent**.
* Only support for some kinds of comment  
Comment can appear anywhere in source code, is it difficult to support all kinds of comment, as comment info does not exists on AST node. On the other hand, I don't think some tricky comment is really need. The following source with comment, which comment disappeared means that it is not supported by rfmt now.
```
// aaaaa

// bbbbb
struct A { // ccccc-DISAPPEARED
    // ddddd
    a: bool, // eeeee
    b: i32, // ffff
    // ggggg
} // hhhhh

// iiiii
fn f(a: bool, /* jjjjj-DISAPPEARED */ b: i32, /* kkkkk-DISAPPEARED */) -> bool { // lllll-DISAPPEARED
    // mmmmm
    const b: bool = false;                  // nnnnn
    let mut a = true;       // ooooo
    a = false; // ppppp
    a!();// qqqqq
    a // rrrrr
} // sssss
// ttttt

// uuuuu
```
* rfmt
```
// aaaaa

// bbbbb
struct A {
    // ddddd
    a: bool, // eeeee
    b: i32, // ffff
    // ggggg
} // hhhhh

// iiiii
fn f(a: bool, b: i32) -> bool {
    // mmmmm
    const b: bool = false; // nnnnn
    let mut a = true; // ooooo
    a = false; // ppppp
    a!(); // qqqqq
    a // rrrrr
} // sssss
// ttttt

// uuuuu
```
