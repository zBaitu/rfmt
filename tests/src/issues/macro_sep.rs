#[test]
fn test_from() {
    from_and_cause!(io::Error::new(io::ErrorKind::Other, "other") => Io(..));
}
