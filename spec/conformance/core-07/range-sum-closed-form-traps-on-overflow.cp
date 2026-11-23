// expect: R0202 at 3 (checked)
// rule: 07.9.6
// input: "1000000000000\n"
// A range's closed-form `sum()` traps on overflow exactly as the walk it stands for would.
let m: 1..=1e18 = read()
print((3..m).step(2).sum())
