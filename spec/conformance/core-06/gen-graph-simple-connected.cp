// expect: ok
// gap: stdlib 2026-11-24T05Z
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
domain V = 0..5
@gen fn input():
    let g = gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
    assert V.all(v => g[v].all(e => e.to != v and 1 <= e.w and e.w <= 9))
    print(V.count)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
