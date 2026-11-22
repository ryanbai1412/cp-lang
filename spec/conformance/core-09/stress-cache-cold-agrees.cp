// expect: ok
// rule: 09.3.8
// tool: stress --seed 1
fn linked(n: Int) -> Bool:
    domain V = 0..n
    var d = Dsu<V>.new()
    for i in 1..n: d.unite(V(i - 1), V(i))
    d.same(V(0), V(n - 1))
@alt(linked)
fn linked_slow(n: Int) -> Bool = true
let n: 1..=8 = read()
print(yes(linked(n)))
