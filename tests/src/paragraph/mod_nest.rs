// a
#[test]
pub mod a; // a

pub mod b; // b

mod c {
    /// bbbbbbbbbbbbbbbbbbbbbb
    // abc
    #[test]
    mod e;
    mod d;
    mod g; // g

    type a = bool; // ca
} // c

mod e {
}

mod f {} // aaaaa

// bbbbb
// ccccc
