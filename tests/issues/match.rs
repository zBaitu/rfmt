fn is_async(asyncness: ast::IsAsync) -> bool {
    match asyncness {
        ast::IsAsync::Async{..} => true,
        ast::IsAsync::Async{} => true,
        _ => false,
    }
}
