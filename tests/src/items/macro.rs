 macro_rules! a {
 () => ()
 }


//
// macro_rules! fmt_src {
// ($src:tt, $($value:tt)*) => (format!(src!($src), $($value)*));
// }
//

//
// macro_rules! fmt_src {
// ($src:tt, $($value:tt)*) => ("a")
// }
//

//
// macro_rules! src {
// (HEADER) => (r#"
// pub const HEADER: Header = Header {{
// name: b"{}",
// major: {},
// minor: {},
// revision: {},
// }};
// "#);
// }
//

fn f() -> bool {
    a!{};
}
