extern { type bool; }

extern "C" {
    static a: bool;
    pub fn f<T>(a: bool) -> i32;
}

extern "Rust" {
    static mut a: bool;
}
