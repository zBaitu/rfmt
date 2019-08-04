//type a<'a, T> where T: 'a = bool;
//type a<'a, 'b, 'c> where 'b: 'a, 'c: 'a + 'b = bool;
//type a<T> where T: for<'a> ::iter<bool>::Iterator<A, B=A> + Sized = bool;
//type a<T> where T: Fn(A, B) -> () = bool;
//type a<T> where T: Fn() -> () = bool;
//type a<'a, 'b, T, U> where for<'a> T: Iterator + 'a, U: Option, 'a: 'b = bool;
