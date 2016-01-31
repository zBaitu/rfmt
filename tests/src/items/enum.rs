enum A {
    Dog,
    Cat,
}

struct BA;
struct BB;

enum B {
    A(BA),
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
