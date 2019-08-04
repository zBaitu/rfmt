fn f() {
    static move |a: bool| -> bool { true };
    let add = |x, y| x + y;
    trees.sort_by(|a, b| {
        if a.path.starts_with("self") {
            Ordering::Less
        } else if b.path.starts_with("self") {
            Ordering::Greater
        } else  {
            a.path.cmp(&b.path)
        }
    });
}
