// expect: exit 1
// rule: 06.4.3
// tool: stress --seed 1
// has: "FAIL"
@alt(program)
fn brute():
    let n: 1..=8 = read()
    print(n + 1)
let n: 1..=8 = read()
print(2 * n)
