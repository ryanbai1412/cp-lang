// expect: ok, output "-5"
// input: "-5 7\n"
// rule: 10.2.3
// `[V]-1e9..=1e9` is an array of a range with a negative low end.
domain V = 1..=2
let a = read<[V]-1e9..=1e9>()
print(a[V.first!])
