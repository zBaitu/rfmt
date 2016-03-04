// a
#[test]
pub mod a; // a

pub mod b; // b

mod c {
    // abc
    #[test]
    mod d; // d

    type a = bool; // ca
} // c

mod e {
}

mod f {}
