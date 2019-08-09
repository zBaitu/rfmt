fn foo(input: Option<usize>) -> usize {
    match input {
        Some(x) => return x,
        None => {
            return 0;
        }
    }
}
