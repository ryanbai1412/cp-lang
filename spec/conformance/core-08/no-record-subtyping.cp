// expect: E0302
// rule: 08.4.2
fn f(e: (w: Int)) -> Int = e.w
print(f((w: 1, u: 2))) // <- here
