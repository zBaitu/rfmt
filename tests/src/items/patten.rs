fn f() {
    match a {
        //_ => true

        //a => true

        //ref mut a => true

        //a @ 1...5 => true

        //Some(b) => true
        //Some(..) => true

        //<T as Trait>::CONST => true

        //A { x: bool, y : B { y1, y2 }, ref mut z, .. } => true

        //(a, b) => true

        //box a => true

        //&mut a => true

        //1 => true

        //1...3 => true

        //[a, b, .., d, e] => true
        //[..] => true

        a!() => true
    }
}
