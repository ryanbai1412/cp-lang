// expect: exit 1
// rule: stress.2
// tool: stress --seed 1 --cases 5
// file: stress-reports-a-saved-case-that-still-fails.tests/stress-7.in "1\n"
// has: "FAIL on the saved case"
// A saved `stress-<seed>.in` the program still gets wrong is reported first, as it was saved, before any new round.
@alt(program)
fn brute():
    let n: 1..=8 = read()
    print(n + n)
let n: 1..=8 = read()
print(n + 3)
