default impl A {}

unsafe impl !A {}

impl A for B {}

impl<T> A for B<T> {
    default const a: bool = true;
    type E<T> where T: bool = T;
    existential type Item<T>: Debug;
    fn f(&self) {}
    fn f(&'a mut self) {}
    fn f<T>(self: bool) {}
    a!(true);
}

unsafe impl !A {}
