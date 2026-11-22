// expect: ok, output "2 2\n0"
// rule: 04b.1.5
domain M = 0..2
let es: [M]Int = [5, 3]
fn count<E, T>(xs: [E]T) -> Int:
    var n = 0
    for e in xs.keys: n += 1
    n
var n = 0
for e in es.keys: n += es[e] - es[e] + 1
domain Z = 0..0
let none: [Z]Int = []
print(n, count(es))
print(count(none))
