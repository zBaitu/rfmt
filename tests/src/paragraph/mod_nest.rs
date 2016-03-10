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
    // zzzzzzzzzzzzzzzz
} // c

mod e {
    // eeeeeeeeee
}

mod f {} // aaaaa

// bbbbb
// ccccc
