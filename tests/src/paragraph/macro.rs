macro_rules! a {
    () => (println!("a"));
    ($a: expr) => (println!($a));
    ($a: ident) => ({
        print!("b");
    });
}

fn f() {
    a!();
    a!("a");
    a!("a", "b");
}
