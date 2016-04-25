rFmt ---- Rust source code formatter
===============================

https://github.com/zBaitu/rfmt

Overview
----------
rfmt is a Rust source code formatter. Yes, there is already an official tool [rustfmt](https://github.com/rust-lang-nursery/rustfmt) from [Rust Nursery](https://github.com/rust-lang-nursery). So why write another one?
* rustfmt is great for configurable, but there are still some style that i don't like in my personal taste.
* Write a code formatter for Rust can make me learn Rust more deeply, for example, the AST of Rust.
* For fun: ) 

Install, Build
----------
* Install

```
cargo install rfmt
```
* Build
```
git clone git@github.com:zBaitu/rfmt.git
cargo build --release
```

Usage
----------
```
Usage: rfmt [options] [path]
    If `path` is a dir, rfmt will do action for all files in this dir recursively.
    If `path` is not specified, use the current dir by default.
    If neither `options` nor `path` is specified, rfmt will format source code from stdin.

Options:
    -a, --ast           print the rust original syntax ast debug info
    -c, --check         check exceed lines and trailing white space lines
    -d, --debug         print the rfmt ir debug info
    -o, --overwrite     overwrite the source file
    -v, --version       show version
    -h, --help          show help
```

Running rfmt from your editor(Copy from rustfmt)
----------
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

Features
----------
Comparing to **rustfmt**, there are some main different features from **rfmt**:
* **DO NOT** parse sub module.
* Keep wrap from user input.
* Different align strategy.
* Group `crate`, `use`, `mod`, `attributes` and sort them.
* **DO NOT** format `doc`, `comment`, `string`. You can use the **check** function to show exceed lines and trailing white space lines.
* Provide check, directory recursively, ast dump.
* Nightly features, like `expr?`, `default fn`.

The following part will show such features in detail, with some existing issues from rustfmt.

### **DO NOT** parse sub mod
What happen when you format the following source by rustfmt when you edit on editor.
```
// lib.rs
pub mod a;
pub mod b;
pub mod c;
pub mod d;
...
```
It will parse all sub modules, this is the default action of the Rust parser. But in fact most of such scenario I just want to format only this file that I editing now.  
rfmt use a custom Rust parser, [rSyntax](https://github.com/zBaitu/rsyntax), it is cloned from the libsyntax of Rust. The main difference between rSyntax and Rust libsyntax is that, rSyntax skip sub module parse.  So rfmt can format quickly on editor scenario.
If you want to format all the source code in a project, just specify the project directory as rfmt command argument:
```
rfmt project_dir
```

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
    let _ = u32::from_be(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) |
                         (d as u32) << 0);
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
    let ref_packet = [0xde, 0xf0, 0x12, 0x34, 0x45, 0x67, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
                      0x86, 0xdd];
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
```
fn main() {
    f(123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```
* rustfmt
```
fn main() {
    f(123456789,
      "abcdefg",
      "hijklmn",
      0987654321,
      "opqrst",
      "uvwxyz",
      123456789,
      "abcdefg",
      "hijklmn",
      0987654321,
      "opqrst",
      "uvwxyz");
}
```
* rfmt
```
fn main() {
    f(123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg",
      "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```
I prefer to put parameters on one line as much as possible. This is only for my personal preferences. But another case I really think it is bad looking.
```
fn main() {
    fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff(123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```
* rustfmt
```
fn main() {
    fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff(123456789,
                                                                                      "abcdefg",
                                                                                      "hijklmn",
                                                                                      0987654321,
                                                                                      "opqrst",
                                                                                      "uvwxyz",
                                                                                      123456789,
                                                                                      "abcdefg",
                                                                                      "hijklmn",
                                                                                      0987654321,
                                                                                      "opqrst",
                                                                                      "uvwxyz");
}
```
* rfmt
```
fn main() {
    fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff(123456789,
            "abcdefg", "hijklmn", 0987654321, "opqrst", "uvwxyz", 123456789, "abcdefg",
            "hijklmn", 0987654321, "opqrst", "uvwxyz");
}
```
If the left align position is beyond limit(It is **50** for now), rfmt prefer double indent align to function call align. rfmt make source code left lean, while rustfmt is right lean, I think. An exsiting issue: [rustfmt should avoid rightwards drifting big blocks of code](https://github.com/rust-lang-nursery/rustfmt/issues/439)
````
fn main() {
    let mut arms = variants.iter().enumerate().map(|(i, &(ident, v_span, ref summary))| {
        let i_expr = cx.expr_usize(v_span, i);
        let pat = cx.pat_lit(v_span, i_expr);

        let path = cx.path(v_span, vec![substr.type_ident, ident]);
        let thing = rand_thing(cx, v_span, path, summary, |cx, sp| rand_call(cx, sp));
        cx.arm(v_span, vec!( pat ), thing)
    }).collect::<Vec<ast::Arm> >();
}
````
* rustfmt
```
fn main() {
    let mut arms = variants.iter()
                           .enumerate()
                           .map(|(i, &(ident, v_span, ref summary))| {
                               let i_expr = cx.expr_usize(v_span, i);
                               let pat = cx.pat_lit(v_span, i_expr);

                               let path = cx.path(v_span, vec![substr.type_ident, ident]);
                               let thing = rand_thing(cx,
                                                      v_span,
                                                      path,
                                                      summary,
                                                      |cx, sp| rand_call(cx, sp));
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
        cx.arm(v_span, vec!(pat), thing)
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
rfmt -c main.rs

a.rs
exceed_lines: {2}
trailing_ws_lines: {1, 4}
----------------------------------------
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
    module: Mod {
        inner: Span { lo: BytePos(7), hi: BytePos(19), expn_id: ExpnId(4294967295) },
        items: [
            Item {
                ident: main#0,
                attrs: [],
                id: 4294967295,
                node: Fn(
                    FnDecl {
                    	......
                    }
                }
            }
        ]
    },
    attrs: [],
    config: [],
    span: Span { lo: BytePos(7), hi: BytePos(18), expn_id: ExpnId(4294967295) },
    exported_macros: []
}
----------------------------------------
0: Isolated [
    "// AST"
]
----------------------------------------
```

### Nightly features, like `expr?`, `default fn`
The rSyntax is cloned from Rust nightly(1.10.0-nightly), so it supports the latest language feature.
```
struct A;

impl A {
    default fn f() -> bool { true }
}

fn f() -> Result<bool, String> { Ok() }

fn ff() -> Result<bool, String> {
    f()?
}

fn main() {
    ff();
}
```
* rfmt
```
struct A;

impl A {
    default fn f() -> bool {
        true
    }
}

fn f() -> Result<bool, String> {
    Ok()
}

fn ff() -> Result<bool, String> {
    f()?
}

fn main() {
    ff();
}
```

Drawbacks
----------
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
