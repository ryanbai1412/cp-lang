// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
// `gen<A>()` in a generic fn draws what `A` was written as: a range, a domain,
// a record or tuple of them, passed on through another generic fn.
domain V = 0..5
fn any<A>() -> A = gen<A>()
fn pass<A>() -> A = any<A>()
@gen fn input():
    let n = any<1..=9>()
    let v = any<V>()
    let r = pass<(to: V, w: 1..=9)>()
    let (u, k) = any<(V, 0..3)>()
    assert 1 <= n <= 9 and 1 <= r.w <= 9 and 0 <= k < 3
    let g = gen<Graph<V>>(simple: true)
    assert V.all(x => g.out(x).all(y => y != x))
    print(n + Int(v) + Int(r.to) + Int(u) + k)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
