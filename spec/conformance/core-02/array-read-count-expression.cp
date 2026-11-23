// expect: ok, output "2 3"
// rule: 02.8.3
// input: "3\n1 2\n2 3\n"
// The count in an array read type may be any `Int` expression (rebuild 7, H-DIMS-EXPR).
let n: 1..=10 = read()
domain V = 1..=n
let es = read<[V.count - 1](u: V, v: V)>()
print(es.len, Int(es[1].v))
