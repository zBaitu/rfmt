#![allow(unused_variables)]

fn main() {
    // Tells the rustfmt tool to not format the following element.
    #[rustfmt::skip]
    struct S {}

    // Controls the "cyclomatic complexity" threshold for the clippy tool.
    #[clippy::cyclomatic_complexity = "100"]
    pub fn f() {}
}
