// expect: R0201
// rule: R02.10.2
// input: "3\n"
// `i` reaches `n` in `0..=n`, so no fact proves `n - i` is in `1..=n`: the check stays.
let n: 1..=10 = read()
var walk: List<1..=n> = []
for i in 0..=n: walk.push(n - i)  // <- here
print(n)
