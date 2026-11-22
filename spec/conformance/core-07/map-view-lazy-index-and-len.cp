// expect: ok, output "4 0 20 1 300 2 100 6"
// rule: 07.1.3 07.4.9
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x * 10
let a = [1, 2, 3, 4]
let m = a.map(f)
let n = m.len
let c0 = calls
let v1 = m[1]
let c1 = calls
let v2 = a.map(f).map(x => x * 10)[2]
let c2 = calls
let s = m.sum()
print(n, c0, v1, c1, v2, c2, s, calls)
