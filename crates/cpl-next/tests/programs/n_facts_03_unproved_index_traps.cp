// expect: R0101
// rule: R03.1.4
// input: "3\n"
// `i + 1` reaches `n`, past `[0..n]`'s last position: the key check stays,
// and an `Int` key out of range traps R0101 (core-04/anon-range-array-traps).
let n: 1..=10 = read()
var nxt: [0..n]0..=n = 0
for i in 0..n: nxt[i + 1] = i  // <- here
print(nxt[0])
