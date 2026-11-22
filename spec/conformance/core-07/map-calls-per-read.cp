// expect: ok, output "2\n2"
// rule: 07.1.3 07.2.16 07.4.9 07.5.4 07.7.1
var calls = 0
fn f(x: Int) -> Int:
    calls += 1
    x
let a = [1, 2, 3]
let m = a.map(f)
print(m[0] + m[0])
print(calls)
