//extern { fn a(b: bool, ...); }
//pub unsafe extern "C" fn f((a, b): (bool, i32)) -> bool {}

pub fn fmt(krate: Crate, leading_cmnts: HashMap<Pos, Vec<String>>,
           trailing_cmnts: HashMap<Pos, String>) -> rfmt::Result {
    Formatter::new(leading_cmnts, trailing_cmnts).fmt_crate(krate)
}
