// expect: ok, output "0 9 3 1"
// rule: 07.9.3
// The free `max(s, by: k)` and `min(s, by: k)` are `s.max(by: k)` and `s.min(by: k)`, with no `.max` written in the file: on a generator and a List the closure takes the element and the fold is `?T`.
fn sq(n: Int) -> yields Int:
    for i in 0..n: yield i * i
let xs = [(1, 2), (3, 0)]
print(max(sq(4), by: x => -x)!, min(sq(4), by: x => -x)!, max(xs, by: q => q.0)!.0, min(xs, by: q => q.0)!.0)
