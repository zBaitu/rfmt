fn f() {
    match a {
        ref mut a @ 1...5 => true,
    }
}
