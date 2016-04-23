#[test]
macro_rules! a {
 () => ()
 }

//
// macro_rules! fmt_src {
// ($src:tt, $($value:tt)*) => (format!(src!($src), $($value)*));
// }
//

//
// macro_rules! fmt_src {
// ($src:tt, $($value:tt)*) => ("a")
// }
//

//
// macro_rules! src {
// (HEADER) => (r#"
// pub const HEADER: Header = Header {{
// name: b"{}",
// major: {},
// minor: {},
// revision: {},
// }};
// "#);
// }
//

/*
fn f() -> bool {
    a!{};
}
*/

fn f() {
    thread_local!(static THREAD_RNG_KEY: Rc<RefCell<ThreadRngInner>> = {
        let r = match StdRng::new() {
            Ok(r) => r,
            Err(e) => panic!("could not initialize thread_rng: {}", e)
        };
        let rng = reseeding::ReseedingRng::new(r,
                                               THREAD_RNG_RESEED_THRESHOLD,                      
                                               ThreadRngReseeder);                               
        Rc::new(RefCell::new(rng))
    });
    let a = true;
}
