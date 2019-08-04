fn f() {
    match a {
        [a, b, .., _, e] => true,
        [..] => true,
    }
}
