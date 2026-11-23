// expect: E0316 at 4
// rule: 03.1.3
// input: "3"
// A struct field's bounds are fixed before input is read; `n` is read.
let n = read<1..=5>()
struct S:
    a: 0..=n
print(n)
