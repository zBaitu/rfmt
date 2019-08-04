impl A {
    fn f() {
        match a {
            ast::ExprKind::Block(ref block, ref label) => ExprKind::Block(Box::new(self.trans_block_expr(block, label))),
        }
    }
}
