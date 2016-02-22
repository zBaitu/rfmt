pub unsafe trait Trait<T>: A + B + 'static + Sized { }

pub trait Trait {
    const a: bool;
    type E: Iterator + 'static + Option = B;
    type A = Result;
    unsafe fn f();
    const fn f(self, a: bool);
    fn f(&self);
    fn f(&'a self);
    fn f(self: Iterator);
}
