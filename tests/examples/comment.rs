// aaaaa

// bbbbb
struct A { // ccccc-DISAPPEARED
    // ddddd
    a: bool, // eeeee
    b: i32, // ffff
    // ggggg
} // hhhhh

// iiiii
fn f(a: bool, /* jjjjj-DISAPPEARED */ b: i32, /* kkkkk-DISAPPEARED */) -> bool { // lllll-DISAPPEARED
    // mmmmm
    const b: bool = false;                  // nnnnn
    let mut a = true;       // ooooo
    a = false; // ppppp
    a!();// qqqqq
    a // rrrrr
} // sssss
// ttttt

// uuuuu
