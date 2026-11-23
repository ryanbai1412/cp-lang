// expect: E0203 at 3
// rule: 07.8.1
// tool: check
// has: "List(1..=n)"
// `[1..=n]` is a list of one range, so the error on it names the list of the numbers.
let n = 5
print([1..=n].shuffled())
