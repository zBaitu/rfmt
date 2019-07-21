macro_rules! a {
    () => (println!("a"));
    ($a: expr) => (println!($a));
    ($a: ident) => ({
        print!("b");
    });
}

type a = f::a!{"a"};

/*
macro_rules! a {
    () => (),
}
*/

fn f() {
    let a!() = bool;

    a![ABCDEFG; 8,
    9 && 10];

    a!("abcdefg", a + b);
    ff(a!("abcdefg", a + b));

    a!{"abcdefg", a + b}

    true
}
