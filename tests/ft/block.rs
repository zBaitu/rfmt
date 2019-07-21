fn f() -> bool {
    let a; // aaaaa
    a = true; // 1111111111111111
    if true {} else {} // 222222222222222
    a!(); // bbbbbbbbbbbbb
    true; // ddddddddd
    unsafe loop {
        a = 10;
    }

} // eeeeeeeeeee
