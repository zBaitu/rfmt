fn f() {
    true;
    ::a::b;
    <Vec<T> as SomeTrait>::SomeType;
    !a;
    &a;
    &mut a;
    a + b;
    a <- b;
    a = bool;
    a += 1;
    [0, true, "a"];
    (0, bool);
    [0; 8];
    (0);

    a.b;
    a.0;

    C;
    B {};
    A { a: true, ..Default::default() };

    a[0];
    b["a"];
    1..2;
    ..2;
    1..;
    ..;
    box a;
    a as bool;
    a: bool;
    {};
    { true };

    if true {} else if false {} else {}
    if false {
        false
    } else if true {
        true
    } else {
        "abcdefg"
    }
    if let a = b {} else {}
    if let Some(ref a) = a {
        true
    }


    'label:
    while true {
        break 'label;
    }
    
    while let a = b {
        false
    }

    'label:
    for a in b {
        continue 'label;
    }
    loop {}


    match a {
        aa if y => true,
        bb | cc => false,
    }

    A::A(B);
    ff(0, bool);
    a.f::<i32>(0, bool);

    move |a, b: bool| -> bool {a};

    return;
    return true;

    a!();
}
