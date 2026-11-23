// expect: E0302 at 3
// rule: 03.5.1
fn f(x: Mod<11>) -> Mod<11> = x * 2
let a: Mod<7> = 3
print(f(a))
