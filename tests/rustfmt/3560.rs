struct Foo {
    something: Option<u32>,
    bar: u32,
}

fn main() {
    let a = Foo {
        something: Some(1),
        bar: 2,
    };
    if let Foo { something: Some(something), .. } = a {
        println!("{}", something);
    }
}
