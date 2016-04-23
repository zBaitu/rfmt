//pub unsafe trait Trait<T>: A + B + 'static + Sized { }
pub trait Trait {
    //const a: bool;
    //type E: Iterator + 'static;
    //unsafe fn f();
    //const fn f(self);
    //fn f(&self);
    //fn f(&'a self);
    fn f(self: Iterator);
}
