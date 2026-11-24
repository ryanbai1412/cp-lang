// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
domain V = 1..=4
@gen fn input():
    var seen: [V]Bool = false
    for _ in 0..200:
        seen[gen<V>()] = true
    assert seen.count(x => x) == V.count
    let arc = gen<(to: V, w: 1..=9)>()
    assert 1 <= arc.w <= 9
    let (u, k) = gen<(V, 0..3)>()
    assert 0 <= k < 3
    print(Int(u) + k + Int(arc.to))
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
