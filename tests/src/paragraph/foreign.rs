extern {}

extern  {
    static mut a: bool;
    pub fn f<T>(a: bool) -> i32;
    pub fn ff(a: bool, ...) -> i32;
}

extern "Rust" {
    static a: bool;
}
