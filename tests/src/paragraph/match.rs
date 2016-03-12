fn f() {
    match a {
        _ => true, // aaa
        true | false if a > 0 => true, // bbb
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
        B { x: bool, // zzzzzzzzzzzzzzz
            y : B { y1, // yyyyyyyyyyyyyyyy
                y2 }, ref mut z, .. // xxxxxxxxx
        } => true,

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
