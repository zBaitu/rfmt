fn f() {
   let span_forward = if is_inner { 2 } else { 1 };
   let span_forward = if is_inner { Ok(()) } else { Err(()) };
   let span_forward = if is_inner { OK } else { ERR };
   let span_forward = if is_inner { A } else if true { B } else { ERR };
}
