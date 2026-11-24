// expect: ok
// rule: stress.2
// tool: stress --seed 1 --cases 5
// file: stress-reruns-a-saved-case-first.tests/stress-7.in "3\n"
// has: "the saved case passes; 5 cases agree"
// A saved `stress-<seed>.in` reruns before the new rounds; once the program agrees on it, the report starts by saying so.
@alt(program)
fn brute():
    let n: 1..=8 = read()
    print(n + n)
let n: 1..=8 = read()
print(2 * n)
