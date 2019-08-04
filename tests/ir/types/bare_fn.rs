type a = for<'a, 'b: 'a> unsafe extern "C" fn(T, bool, ...) -> usize;
