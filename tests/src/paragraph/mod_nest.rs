// a
#[test]
pub mod a; // a

pub mod b; // b

mod c {
    // abc
    #[test]
    mod d; // d
    mod e; // e
    mod g; // g

    type a = bool; // ca
} // c

mod e {
}

mod f {}
