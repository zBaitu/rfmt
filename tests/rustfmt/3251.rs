fn foo() {
    let coords: Vec<_> =
        [(self.y - 1, self.x),
         (self.y, self.x - 1),
         (self.y, self.x + 1),
         (self.y + 1, self.x)].iter()
                              .filter_map({})
                              .filter({})
                              .map({})
                              .collect();
}
