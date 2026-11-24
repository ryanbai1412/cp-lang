// expect: E0704
// rule: 08.4.4
fn f<R: (u: Int, ..)>(x: R) -> Int = x.u
struct S: u: Int
print(f(S(1))) // <- here
