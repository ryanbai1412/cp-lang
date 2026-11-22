// expect: ok, output "0\n3 3\n2"
// rule: 06.3.1 07.2.10 07.3.1 07.6.2
var calls = 0
fn gen(n: Int) -> yields Int:
    calls += 1
    for i in 0..n: yield i
let g = gen(3)
print(calls)
print(g.sum(), g.sum())
print(calls)
