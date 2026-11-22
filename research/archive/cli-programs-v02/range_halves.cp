domain V = 0..6
let (l0, r0) = (V(0)..V(0)).halves()
assert l0.is_empty and r0.is_empty

let (l1, r1) = (V(2)..=V(2)).halves()
assert l1.len == 1 and r1.is_empty and l1.first == V(2)

let (l2, r2) = (V(2)..=V(3)).halves()
assert l2.len == 1 and r2.len == 1 and l2.first == V(2) and r2.first == V(3)

let (l5, r5) = (V(1)..=V(5)).halves()
assert l5.len == 3 and r5.len == 2
assert l5.start == V(1).l and l5.end == V(4).l and r5.start == V(4).l and r5.end == V(5).r

let (il, ir) = (3..8).halves()
assert il.len == 3 and ir.len == 2 and il.start == 3 and il.end == 6 and ir.start == 6 and ir.end == 8

let (el, er) = (5..2).halves()
assert el.is_empty and er.is_empty

fn sum_dc(r: Range<V>, a: [V]Int) -> Int:
    if r.is_empty: return 0
    if r.len == 1: return a[r.first!]
    let (left, right) = r.halves()
    sum_dc(left, a) + sum_dc(right, a)

let a: [V]Int = [V] i => (i.to_idx + 1) * 10
print(sum_dc(V.first.l..V.last.r, a), il.len, ir.len)
