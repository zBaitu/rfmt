impl A {
fn build() -> Client {
    let compressor = if let Some(compression) = self.compression {
        Some(compression::build(compression))
    } else {
        None
    };

    if let Some(compression) = self.compression {
        Some(compression::build(compression))
    } else {
        None
    }
}
}
