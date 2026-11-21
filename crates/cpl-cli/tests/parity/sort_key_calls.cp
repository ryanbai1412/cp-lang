var calls = 0
fn key(x: Int) -> Int:
    calls += 1
    print("key", x)
    100 / x

var xs = [5, 3, 9, 1, 4, 8, 2, 7]
xs.sort(by: x => key(x))
print(xs, calls)
calls = 0
let ys = sorted([4, 1, 3, 2, 5], by: x => key(x), desc: true)
print(ys, calls)
calls = 0
var ps = [(2, "b"), (1, "a"), (2, "a"), (1, "b")]
ps.sort(by: p => key(p.0))
print(ps, calls)
calls = 0
var one = [6]
one.sort(by: x => key(x))
let none_at_all = sorted([0][1..], by: x => key(x))
print(one, none_at_all.len, calls)
var fixed: [0..4]Int = [3, 1, 4, 2]
fixed.sort(by: x => key(x), desc: true)
print(fixed, calls)
