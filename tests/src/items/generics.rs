type a<'a> = bool;
type a<'a, 'b: 'a, 'c: 'a + 'b> = bool;
type a<T> = bool;
//type a<T=u8> = bool;
//type a<'a, T: 'a> = bool;
//type a<T: Sized> = bool;
//type a<T: ?Sized> = bool;
