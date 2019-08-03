extern {}

extern  {
    static mut a: bool; // a
    pub fn f<T>(a: bool) -> i32 where T: bool; // b
    pub fn ff(a: bool, ...) -> i32; // c
    // end
} // aaaaaaaaaaa

extern "Rust" {
    static a: bool;
}
