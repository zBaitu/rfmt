type a = bool;
type a = result::Result;
type a = <::i32>::MAX;
type a = <::i32 as a::b::Vec<bool>>::c::d::MAX;

type a = *const bool;
type a = *mut bool;
type a = &bool;
type a = &'a mut bool;

type a = [bool];
type a = [[bool]];
type a = [bool; 8];

type a = &[bool];
type a = &'a bool;

type a = ();
type a = (bool);
type a = (bool, usize);
type a = (bool, usize, isize, String, Vec, str);

//type a = for<'a> unsafe extern "C" fn(bool) -> usize;
type a = for<'a> extern "Rust" fn(bool) -> usize;
type a = for<'a> unsafe extern "C" fn(bool) -> usize;
type a = for<'a> fn(bool) -> usize;

type a = Result + Iterator<T> + 'static + Sized;

type a = for<'a, 'b: 'a> Foo<&'a Bar>;

type a = a!("a");

type a = _; // a


// b
