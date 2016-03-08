type a<T> = Option<T>;

type a<'a, 'b,
'c: 'a + 'b, 
T: 'a + for<'b> ::iter::Iterator, U, V, R = Result> where 'b: 'a, U: Eq , V: Fn(bool, i32) -> String = Option<'a, T, U  = Result>;

type a<T> where T: Fn(bool) -> u32  = Option<'a, T, U  = Result,>;
type a<T: Fn(bool) -> u32> = Option<'a, T, U  = Result,>;
type a<'a, 'b> where 'a: 'b = Option<T>;
//type a<T> where for<'a: 'b> = Option<T>;
type a<T> where for<'a: 'b> T: Iterator + 'a = Option<T>;
