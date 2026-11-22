// expect: ok, output "8"
// rule: 06.3.5 07.6.4
fn sq(n: Int) -> yields Int:
    for i in 1..=n: yield i * i
let a = List(sq(3))
print(zip(a, [1, 2, 3]).map((x, y) => x - y).sum())
