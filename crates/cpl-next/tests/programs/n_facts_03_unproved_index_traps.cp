// expect: R0102
// rule: R03.1.4
// input: "3\n"
// `i + 1` reaches `n`, past `[0..n]`'s last position: the domain check stays.
let n: 1..=10 = read()
var nxt: [0..n]0..=n = 0
for i in 0..n: nxt[i + 1] = i  // <- here
print(nxt[0])
