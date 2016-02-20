pub fn f() {}

pub unsafe extern "C" fn ff<T>((a, b): (bool, i32)) -> Result<String, bool> where T: Iterator  {}
