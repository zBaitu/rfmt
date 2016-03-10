impl A{}
impl<T> !A for B{}
pub unsafe impl A for B{}
impl<T> A for B<T> 
where T: 'static + Iterator + Option {
    const a: bool = true; // a
    type E = T; // b
    fn f(&self) {
        println!("hello world"); // c
    } // d
    a!(ABCD); // e
}
