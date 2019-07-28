macro_rules! a { () => (); }

macro_rules! m {
    (1) => {};
}

macro_rules! ambiguity {
    ($($i:ident)* $j:ident) => {};
}

macro_rules! d {
    ($arg:expr) => ({println!("{:#?}", $arg)});
}

macro_rules! p {
    () => ({println!()});
    ($arg:expr) => ({println!("{}", $arg)});
    ($fmt:expr, $($arg:tt)*) => ({println!($fmt, $($arg)*)});
    ($($arg:tt)+) => ({println!("{}", $($arg)+)});
}


a!();
a![];
a!{}
println!("{}");
println!(a, b);
maybe_wrap!( self, " ", "", expr, fmt_expr);
