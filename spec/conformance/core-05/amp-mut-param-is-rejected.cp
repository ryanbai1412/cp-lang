// expect: E0503
// rule: 05.2.3
// has: "mut x: T"
// There are no `&mut` types: a parameter that changes the caller's place is `mut`.
fn bump(x: &mut Int): // <- here
    x += 1
var n = 0
bump(mut n)
print(n)
