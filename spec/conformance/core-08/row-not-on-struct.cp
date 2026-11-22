// expect: E0704 at 3
// rule: 08.4.4
fn f<R: (u: Int, ..)>(x: R) -> Int = x.u
struct S: u: Int
print(f(S(1)))
