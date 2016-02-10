pub struct Typesetter {
    s: String,
}

impl Typesetter {
    pub fn new() -> Typesetter {
        Typesetter {
            s: String::new(),
        }
    }

    pub fn string(self) -> String {
        self.s
    }

    pub fn tmp(&mut self) {}
}
