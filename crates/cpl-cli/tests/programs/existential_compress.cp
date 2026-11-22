fn compress_vals<D>(xs: [D]Int) -> (domain C, [D]C, [C]Int):
    var uniq: List<Int> = []
    for i in D:
        uniq.push(xs[i])
    uniq.sort()
    uniq = uniq.deduped()
    domain C = 0..uniq.len
    let idx: [D]C = [D] i => C((0..uniq.len).first_true(k => uniq[k] >= xs[i])!)
    let vals: [C]Int = [C] c => uniq[c.ord]
    (C, idx, vals)

domain I = 0..5
let a: [I]Int = [30, 10, 30, 20, 10]
let (domain V, mapped, vals) = compress_vals(a)
print(V.len, mapped[I(0)].ord, mapped[I(1)].ord, mapped[I(3)].ord, vals[mapped[I(3)]])

domain J = 1..=3
let b: [J]Int = [99, 99, -5]
let (domain W, mapped2, vals2) = compress_vals(b)
print(W.len, mapped2[J(1)].ord, mapped2[J(3)].ord, vals2[mapped2[J(1)]])
