// expect: ok, output "3 0 3 0"
// rule: 07.1.3 07.4.9
// A piece of a map view keeps the source's length, so `.len` of `m.drop(1)` or `m.rev` calls `f` on nothing (A7).
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x * 10
let m = [1, 2, 3, 4].map(f)
let n = m.drop(1).len
let c0 = calls
let r = m.rev.take(3).len
print(n, c0, r, calls)
