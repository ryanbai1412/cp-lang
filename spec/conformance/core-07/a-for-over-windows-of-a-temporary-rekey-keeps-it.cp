// expect: ok, output "3 1\n1 4\n1 3\n3 4\n9"
// rule: 07.5.10
// A `rekey` of a fresh list owns its cells, and the windows over it point into
// them: the `for` header keeps that temporary for the whole loop.
let xs = [3, 1, 4]
for q in [3, 1, 4].rekey(xs.keys).windows(2): print(List(q))
for q in xs.sorted().rekey(1..=3).windows(2): print(List(q))
fn keyed(xs: List<Int>) -> Int:
    var s = 0
    for q in [x for x in xs].rekey(xs.keys).windows(2): s += q.sum()
    s
print(keyed(xs))
