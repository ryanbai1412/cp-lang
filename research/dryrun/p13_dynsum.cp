// CSES 1648 Dynamic Range Sum Queries: point assignment, range sum, on a Fenwick tree.
let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
var x: [I]Int = read()
var bit = Fenwick<I, Int>(x)
repeat q:
    let (kind, a, b): (1..=2, Int, Int) = read()
    if kind == 1:
        let k = I(a)
        bit.add(k, b - x[k])
        x[k] = b
    else:
        print(bit.sum(I(a)..=I(b)))
