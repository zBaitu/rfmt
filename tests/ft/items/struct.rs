struct A;

struct B {}

struct Tuple(bool, i32);

pub struct Point<T> where T : bool {
    #[test]
    pub x: i32, // a
    y: i32,
}

