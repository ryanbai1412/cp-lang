// expect: E0316
// rule: 03.1.3
// input: "3"
// A struct field's bounds are fixed before input is read; `n` is read.
let n = read<1..=5>()
struct S:
    a: 0..=n // <- here
print(n)
