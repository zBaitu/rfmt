pub unsafe trait Trait<T>: A + B + 'static + Sized { } // aaaaa

pub trait Trait {
    // aaaaaa
    const a: bool; // bbbbb
    type E: Iterator + 'static + Option = B;
    type A = Result;
    unsafe fn f();
    const fn f(self, a: bool);
    fn f(&self);
    fn f(&'a self);
    fn f(self: Iterator); // cccccc
} // aaaaaaaaaaaa

// a
// b
