//
// aa
impl<T> A for B<T> { // impl-trailing
    default const a: bool = true; // aa
    type E = T; // bb
    existential type Item: Debug;
    fn f(&self) {}
    a!(true);
    // impl-leading
} // dd

