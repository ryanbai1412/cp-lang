// expect: ok, output "3 2\n2"
// rule: 07.7.3
// A generator whose own helper fns write its locals still stops pulling at `take(k)` and at a
// `break`: the values it never makes cost nothing (R07.1.3).
var made = 0
fn halves(n: Int) -> yields Int:
    var todo: List<Int> = []
    fn add(x: Int):
        made += 1
        todo.push(x)
    add(n)
    while let x = todo.pop():
        yield x
        if x > 1: add(x / 2)
print(halves(1_000_000).take(2).sum() - 1_500_000 + 3, made)
made = 0
for x in halves(1_000_000):
    if x < 1_000_000: break
print(made)
