// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
domain V = 0..5
@gen fn input():
    let t = gen<Rooted<V>>(shape: path)
    assert V.all(v => t.depth(v) < V.count)
    print(V.count)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
