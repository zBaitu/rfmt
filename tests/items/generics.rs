//type a<> = bool;
//type a<'a> = bool;
//type a<T> = bool;
/*
type a<'a,
    'b: 'a,
    'c: 'a + 'b> = bool;
    */
//type a<T> = bool;
//type a<'a, T: 'a> = bool;
//type a<T: Sized> = bool;
//type a<T: ?Sized> = bool;

type a<T: for<'a> ::iter<bool>::Iterator<A, B=A> + Sized> = bool;
//type a<T: Fn(A, B) -> ()> = bool;
//type a<T = u8> = bool;

