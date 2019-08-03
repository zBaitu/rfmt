auto trait IsCool {}

unsafe trait Trait<T>: A + B + 'static + Sized where T: bool {}

pub trait Trait {
    const a: bool = true;
    type E<T>: Iterator + 'static where T: bool  = bool;
    unsafe fn f();
    const fn f(&self);
    fn f(&self);
    fn f(&'a self);
    fn f(self: Iterator);
    fn f<T>(&self) where T: bool;
    a!(true);
}
