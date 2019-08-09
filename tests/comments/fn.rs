//
// aa
pub fn fmt(krate: Crate, leading_cmnts: HashMap<Pos, Vec<String>>,
           trailing_cmnts: HashMap<Pos, String>) -> rfmt::Result { // fn-trailing
    // aa
    Formatter::new(leading_cmnts, trailing_cmnts).fmt_crate(krate) // bb
    // fn-leading
} // dd
// ee
//
