fn main() {
    let mut arms = variants.iter().enumerate().map(|(i, &(ident, v_span, ref summary))| {
        let i_expr = cx.expr_usize(v_span, i);
        let pat = cx.pat_lit(v_span, i_expr);

        let path = cx.path(v_span, vec![substr.type_ident, ident]);
        let thing = rand_thing(cx, v_span, path, summary, |cx, sp| rand_call(cx, sp));
        cx.arm(v_span, vec![ pat ], thing)
    }).collect::<Vec<ast::Arm> >();
}
