fn f() {
    match a {
        A { x: bool, y: B { y1, y2, }, ref mut z, .. } => true,
        A {
            xxxxxxxxxxxxxxxxxxxx: bool, // aa
            // bb
            yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy: B { y1, y2, },
            ref mut zzzzzzzzzzzzzzzzzzzzz,
            // cc
            ..
        } => true,
    }
}
