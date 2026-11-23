// A read-only `[V]T` parameter takes a view over `V` as it is (R07.5.11).
domain V = 1..=5
fn total(xs: [V]Int) -> Int = xs.sum()
fn first_key(xs: [V]Int) -> Int = Int(xs.keys.first!)
fn at(xs: [V]Int, v: V) -> Int = xs[v]
fn walk(xs: [V]Int) -> Int:
    var s = 0
    for v in xs.keys:
        s += Int(v) * xs[v]
    s
let a = [V] v => Int(v) * 10
let w = a[V(2)..V(5)]
print(total(a), total(w), total(a[V(3)..]))
print(first_key(a), first_key(w), at(w, V(3)), w.len)
print(walk(a), walk(w))
