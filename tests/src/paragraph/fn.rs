pub fn f() {}

pub unsafe extern "C" fn fff<T>((a, b): (bool, i32)) -> Result<String, bool> where T: Iterator  {}

mod a {
    pub unsafe extern "C" fn fff<T>((a, b): (bool, i32)) -> Result<String, bool> where T: Iterator  {}
}
