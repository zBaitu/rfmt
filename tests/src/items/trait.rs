//pub unsafe trait Trait<T>: A + B + 'static + Sized { }
pub trait Trait {
    //const a: bool;
    type E: Iterator + 'static;
    //fn f();
    //fn f(&self);
}
