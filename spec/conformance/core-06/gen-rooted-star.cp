// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
// `gen<Rooted<V>>(shape:)` is `Rooted<V>.gen(shape:)`: the shape is a `TreeShape`, random by default.
domain V = 0..6
@gen fn input():
    let s = gen<Rooted<V>>(shape: Star)
    let t = gen<Rooted<V>>()
    assert V.all(v => s.depth(v) <= 1)
    assert V.all(v => t.depth(v) < V.count)
    print(V.count)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
