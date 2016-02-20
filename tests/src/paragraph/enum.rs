enum A {
    Dog = 10,
    Cat,
}

struct BA;
struct BB;

enum B {
    A(BA, 
    BB),
    B(BB),
}

enum C {
    CA {
        a: bool,
    },
    CB {
        b: i32,
    }
}

enum E {}
