//extern { fn a(b: bool, ...); }
pub unsafe extern "C" fn f((a, b): (bool, i32)) -> bool {}
