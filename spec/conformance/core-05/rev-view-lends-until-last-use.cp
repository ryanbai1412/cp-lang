// expect: E0602 at 4
// rule: 05.7.4
// `a.rev` is a view: `a` stays lent to `w` until `w`'s last use, so the push is rejected.
var a = [1, 2, 3]
let w = a.rev
a.push(4)
print(w.nth(0))
