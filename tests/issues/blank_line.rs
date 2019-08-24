extern crate a;
extern crate b;

extern crate c;
use b;
use c;



use a;


fn a() {
}
fn b() {
}

struct A {
}
struct B {
}

pub trait Trait {
    const a: bool = true;
    type E: Iterator + 'static = bool;
    unsafe fn f();
    const fn f(&self);
    fn f(&self) {}
    fn f(&'a self);
    fn f(self: Iterator) {}
    a!(true);
}

impl A {
    fn a() {}
    fn b() {}
}
