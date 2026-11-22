// expect: ok
// rule: 06.4.3
// tool: stress --seed 1 --cases 50
// has: "cases agree"
@gen fn input():
    let x = gen(1..=8)
    assert 1 <= x and x <= 8
    print(x)
@alt(program)
fn brute():
    let k: Int = read()
    print(k)
let k: Int = read()
print(k)
