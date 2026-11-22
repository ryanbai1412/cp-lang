// expect: E0203 at 1
// rule: 08.4.4 12.11.4
fn f<R: (u: Int, ..)>(x: R) -> Int = x.w
