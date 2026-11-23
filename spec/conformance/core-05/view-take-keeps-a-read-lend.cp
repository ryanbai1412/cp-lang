// expect: ok, output "10\n30 20\n20 30\n20 30"
// rule: 05.7.6
// `r.take(k)` only reads `r`, so a view of a `let` stays a read lend and `a` can still be read while `r` is live; `r.drop(k)` already does (A7).
let a = [10, 20, 30]
let r = a.rev
print(a[0])
print(List(r.take(2)))
let s = a[0..3]
print(List(a[1..3]))
print(List(s.take(1)).len + 19, s.last!)
