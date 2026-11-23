domain V = 1..=4
let a = [V] v => 10 * Int(v)
fn by_cut<D>(xs: [D]Int) -> [D.cuts]Int:
    var p: [D.cuts]Int = 0
    for (i, x) in xs.items:
        p[i.r] = x
    p
fn walked<D>(xs: [D]Int) -> List<Int> = [Int(i) for (i, _) in xs.items]
let q = by_cut(a.rev)
print(q[V(1).r], q[V(4).r], q[V(1).l])
print(walked(a.rev))
