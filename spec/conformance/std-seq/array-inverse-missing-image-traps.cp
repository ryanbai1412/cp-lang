// expect: R0001
// A total `[D]E` whose images miss a value of `E` is no bijection: `inverse` traps.
let c: [0..3]0..4 = [2, 0, 1]
print(c.inverse()[0]) // <- here
