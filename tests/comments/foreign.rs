//
// aa
extern "C" { // foreign-trailing
    static a: bool; // aa
    pub fn f<T>(a: bool) -> i32; // bb
    // foreign-leading
} // cc
// dd
//
