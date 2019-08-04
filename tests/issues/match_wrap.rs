impl A {
    fn f() {
        match a {
            ast::ExprKind::Block(ref block, ref label) => ExprKind::Block(Box::new(self.trans_block_expr(block, label))),
            ast::TyKind::Rptr(ref lifetime, ref mut_type) => {
                TypeKind::Ref(Box::new(self.trans_ref_type(lifetime, mut_type)))
            }
        }
    }
}
