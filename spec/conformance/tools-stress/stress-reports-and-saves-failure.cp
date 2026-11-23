// expect: exit 1
// rule: stress.1
// tool: stress --seed 1
// has: "first difference at line 1"
// has: "stress-1.in"
// lacks: "error["
@alt(program)
fn brute():
    let n: 1..=8 = read()
    print(n + 1)
let n: 1..=8 = read()
print(2 * n)
