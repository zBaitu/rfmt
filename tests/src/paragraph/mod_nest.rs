// a
#[test]
pub mod a;

pub mod b;

mod c {
    // abc
    #[test]
    mod d;
}

mod e {
}

mod f {}
