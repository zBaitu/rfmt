fn f() {
    trees.sort_by(|a, b| {
        if a.path.starts_with("self") {
            Ordering::Less
        } else if b.path.starts_with("self") {
            Ordering::Greater
        } else {
            a.path.cmp(&b.path)
        }
    });
}
