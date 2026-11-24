// expect: R0101 (checked)
// rule: 07.4.5
// input: "0\n"
// A key that isn't one of a slice's keys traps naming them, and never reads the source around the slice (decision 388).
let a = [1, 2, 3, 4, 5]
let w = a[1..4]
print(w[read<Int>()]) // <- here
