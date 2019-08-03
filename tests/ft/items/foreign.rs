extern { type bool; }

//extern { a!(true); }

extern "C" {
    static a: bool;
    pub fn f<T>(a: bool) -> i32 where T: bool;
}

extern "Rust" {
    static mut a: bool;
}
