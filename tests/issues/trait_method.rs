pub trait Message {
    fn f<T>() -> bool where T: Sized;
    fn new() 
        -> Self where Self: Sized;
}
