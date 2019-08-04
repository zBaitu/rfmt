auto trait IsCool {}

unsafe trait Trait<T>: A + B + 'static + Sized {}

pub trait Trait {
    const a: bool = true;
    type E: Iterator + 'static = bool;
    unsafe fn f();
    const fn f(&self);
    fn f(&self);
    fn f(&'a self);
    fn f(self: Iterator);
    a!(true);
}
