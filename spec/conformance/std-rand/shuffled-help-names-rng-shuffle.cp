// expect: E0203
// rule: 07.8.1
// tool: check
// has: "rng.shuffle(mut ys)"
// There is no `shuffled`; the help names the real call, `shuffle` on an `Rng` (std.rand).
let ys = List(1..=5)
print(ys.shuffled()) // <- here
