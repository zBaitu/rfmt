enum A {
    Dog = 10, // a
    Cat, // b
} // c

struct BA;
struct BB;

enum B {
    A(BA, 
    BB),
    B(BB),
}

enum C {
    // ccccccccccccc
    CA {
        a: bool, // aaaaaaaaaaaa
    },
    // ddddddddddddddd
    CB {
        b: i32, // bbbbbbbbbbbbb
    }
}

enum E {}
