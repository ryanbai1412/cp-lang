// expect: ok, output "2 3 4\n2 3 4\n6\n9 9"
// rule: 07.4.9
// A map view calls `f` on every read, so reading it whole twice calls `f` twice per element; an optimization may only cache an `f` with no effects.
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x + 1
let a = [1, 2, 3]
let m = a.map(f)
print(m)
print(m)
print(calls)
let s = m.sum()
print(s, calls)
