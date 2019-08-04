fn f() {
    match a {
        Some(b) => true,
        Some(..) => true,
        Some(a, .., b) => true,
    }
}
