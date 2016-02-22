impl A{}
impl<T> !A for B{}
pub unsafe impl A for B{}
impl<T> A for B<T> {
    const a: bool = true;
    type E = T;
    fn f(&self) {}
    a!();
}
