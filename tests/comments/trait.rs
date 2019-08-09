// 
// aa
pub trait Trait { // trait-trailing
    const a: bool = true; // aa
    type E: Iterator + 'static = bool; // bb
    unsafe fn f();
    const fn f(&self);
    fn f(&self);
    fn f(&'a self);
    fn f(self: Iterator);
    a!(true);
    // trait-leading
}
