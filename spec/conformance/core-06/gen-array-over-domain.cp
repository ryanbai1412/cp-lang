// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
domain V = 0..5
@gen fn input():
    let a = gen<[V]1..=1e9>()
    assert a.len == 5 and a.all(x => 1 <= x and x <= 1e9)
    print(a.len)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
