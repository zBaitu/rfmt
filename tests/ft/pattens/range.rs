fn f() {
    match a {
        0..1 => true,
        0...1 => true,
        0..=1 => true,
    }
}
