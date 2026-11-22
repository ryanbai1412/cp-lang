// expect: ok, output "3"
// rule: 12.11.4
fn f<R: (u: Int, ..)>(x: R) -> Int = x.u
print(f((u: 3, w: 4)))
