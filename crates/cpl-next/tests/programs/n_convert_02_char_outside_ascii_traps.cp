// expect: R0201
// rule: R02.1
// input: "200"
// `Char(k)` traps where `k` is not a code `0..=127`, in both engines.
let k = read<Int>()
print(Char(k - 100))
print(Char(k)) // <- here
