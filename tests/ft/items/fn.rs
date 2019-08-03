async unsafe fn f() {}

const fn f() {}

extern "C" fn f() {}

pub fn fmt<T>(krate: Crate, leading_cmnts: HashMap<Pos, Vec<String>>,
           trailing_cmnts: HashMap<Pos, String>) -> rfmt::Result where T: bool {
    Formatter::new(leading_cmnts, trailing_cmnts).fmt_crate(krate)
}

fn f(a: _) {}
