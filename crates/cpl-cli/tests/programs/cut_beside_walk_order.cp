domain V = 1..=4
let a = [V] v => 10 * Int(v)
fn cuts<D>(xs: [D]Int) -> List<(Int, Int)> = [(Int(xs.cut_before(i)), Int(xs.cut_after(i))) for (i, _) in xs.items]
fn suffix<D>(xs: [D]Int) -> [D.cuts]Int:
    var p: [D.cuts]Int = 0
    var acc = 0
    for (i, x) in xs.items:
        acc += x
        p[xs.cut_after(i)] = acc
    p
print(cuts(a))
print(cuts(a.rev))
print(cuts(a[V(2)..=V(3)].rev))
print(cuts(a[V(3)..=V(3)].rev))
print([suffix(a)[c] for c in V.cuts])
print([suffix(a.rev)[c] for c in V.cuts])
let l = [5, 6, 7]
print(l.cut_after(0), l.rev.cut_after(0), l[1..].rev.cut_before(2), "abc".rev.cut_after(2))
