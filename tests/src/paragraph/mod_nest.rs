// a
#[test]
pub mod a;

pub mod b;

mod c {
    // abc
    #[test]
    mod d;

    type a = bool;
}

mod e {
}

mod f {}
