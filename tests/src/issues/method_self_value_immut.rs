impl A {
    pub fn result(self) -> rfmt::Result {
        rfmt::Result {
            s: self.s,
            exceed_lines: self.exceed_lines,
            trailing_ws_lines: self.trailing_ws_lines,
        }
    }
}
