// CF 1916E Happy Life in University. Walk w bottom-up keeping, at each
// vertex u of w's subtree, diff(u, w) in a max segment tree over the Euler
// order: entering w adds 1 to its subtree, except below the nearest
// descendants x with the same activity (they already count it). The answer
// at w is the product of the two largest child-subtree maxima (1 stands for
// w itself).
@!multitest
let n: 1..=3e5 = read()
domain V = 1..=n
let p = read<[n - 1]V>()
let col = read<[V]1..=n>()
let par: [V]?V = [V] v => if Int(v) == 1: None else: Some(p[Int(v) - 2])
let t = Rooted.from_parents(par)
// near[u]: the closest descendants of u with u's activity.
var near: [V]List<V> = []
var top: List<?V> = List.filled(n + 1, None)
var saved: [V]?V = None
for e in t.tour:
    match e:
        Enter(v, _) =>
            let c = col[v]
            saved[v] = top[c]
            if let u = top[c]: near[u].push(v)
            top[c] = Some(v)
        Exit(v, _) => top[col[v]] = saved[v]
let zero: [V]Int = 0
var seg = SegTree(max, t.reorder(zero), act: add)
var ans = 1
for w in t.order.rev:
    seg.apply(t.sub(w), 1)
    for x in near[w]: seg.apply(t.sub(x), -1)
    var t1 = 1
    var t2 = 1
    for c in t.children(w):
        let m = seg.fold(t.sub(c))
        if m > t1:
            t2 = t1
            t1 = m
        elif m > t2:
            t2 = m
    ans max= t1 * t2
print(ans)
