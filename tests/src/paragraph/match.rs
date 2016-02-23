fn f() {
    match a {
        _ => true,
        true | false if a > 0 => true,
        1...3 => true,
        a @ 1...5 => true,

        a => true,

        ref mut a => true,

        &mut a => true,


        Some(a, ref b) => true,
        Some(..) => true,
        Some(_) => true,

        <T as Trait>::CONST => true,

        A {} => false,
        B { x: bool, y : B { y1, y2 }, ref mut z, .. } => true,

        [a, b, .., d, e] => true,
        [.., a] => true,
        [a, ..] => true,
        [..] => true,

        () => false,
        (a, b) => true,

        box a => true,


        1 => true,



        a!() => true,
    }
}
