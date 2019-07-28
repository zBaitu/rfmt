fn f() {
    true;

    a::b;
    <::i32 as Vec<bool>>::MAX;

    &a;
    &mut b;

    !a;

    f()?;

    a + b;
    a = bool;
    a += 1;
    a + b + c;

    [true; 10];

    [true, 1, "a"];

    (true, 1, "a");

    a = b[1];

    NothingInMe {};
    Point { x: 10.0, y: 20.0 };
    TuplePoint { 0: 10.0, 1: 20.0 };
    Point3d { y: 0, z: 10, ..base };

    a.a;
    a.0;

    a: bool;

    a as bool;

    1..2;
    1..;
    ..2;
    1..=2;
    ..=2;

    {}
    'label: {}
    'label: {
        true;
    }

    if true {} else if false {} else {}
    if true {
        a;
    } else if false {
        b;
    } else {
        c;
    }

    if let Some(ref a) = a {
        true;
    }
    if let a | b | c = d {} else {}

    'label: while true {}

    while let a | b | c = d {}

    'label: for a in b {}

    'label: loop {}

    'label: for a in b {
        break 'label;
    }

    let a = for b in c {
        break 0;
    };

    'label: for a in b {
        continue 'label;
    }

    ff(0, bool);
    Position(0, 0, 0);

    a.f::<i32>(0, bool);
    ::a.b.c(0, 1);

    static move |a: bool| -> bool { true; false; };
    let add = |x, y| x + y;

    return bool;
    return;
}
