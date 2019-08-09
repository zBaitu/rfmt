lazy_static! {
    pub static ref BLOCKING_POOL: tokio_threadpool::ThreadPool = {
        tokio_threadpool::Builder::new().pool_size(1).build()
    };

    static ref FOO: Foo = unsafe {
        very_long_function_name().another_function_with_really_long_name()
    };
}
