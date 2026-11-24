// The windows of a temporary `rekey` point into the cells it owns: the `for`
// keeps it for the whole loop.
fn keyed(xs: List<Int>) -> Int:
    var s = 0
    for q in List(xs).rekey(xs.keys).windows(2): s += q.sum()
    s
let n: 1..=100 = read()
let xs = [i * 7 % 5 for i in 0..n]
for q in List(xs).rekey(xs.keys).windows(2): print(List(q))
for q in xs.sorted().rekey(1..=n).windows(3): print(List(q))
print(keyed(xs))
