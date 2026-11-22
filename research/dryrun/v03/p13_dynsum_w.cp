// CSES 1648 Dynamic Range Sum Queries: point assignment, range sum, on a Fenwick tree.
let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
var x = read<[I]Int>()
var fw = Fenwick(x)
for _ in 0..q:
    let (kind, a, b) = read<(1..=2, Int, Int)>()
    if kind == 1:
        let k = I(a)
        fw.add(k, b - x[k])
        x[k] = b
    else:
        print(fw.prefix(I(b).r) - fw.prefix(I(a).l))
