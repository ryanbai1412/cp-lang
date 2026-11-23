// expect: ok
// rule: stress.1
// tool: stress --seed 1 --cases 50
// has: "cases agree"
@alt(program)
fn brute():
    let n: 1..=8 = read()
    print(n + n)
let n: 1..=8 = read()
print(2 * n)
