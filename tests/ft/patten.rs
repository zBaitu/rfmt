fn f() {
    match a {
        _ => true,

        1..=3 => true,
        1..2 => true,

        &mut a => true,

        <T as Trait>::CONST => true,
        a::CONST => true,

        a => true,
        ref mut a => true,
        a @ 1...5 => true,

        A { x: bool, y : B { y1, y2 }, ref mut z, .. } => true,

        Some => true,
        Some(b) => true,
        Some(x, y) => true,
        Some(.., a) => true,
        Some(a, .., b) => true,
        Some(a, ..) => true,

        (a) => true,
        (a, b) => true,
        (a, .., b) => true,
        (.., b) => true,
        (a, ..) => true,

        [a, b, c] => true,
        [a, i.., e] => true,
        [a, b, .., d, e] => true,
        [..] => true,

        //a!() => true
    }
}
