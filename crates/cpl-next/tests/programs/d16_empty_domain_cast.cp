// expect: R0102
// input: "0\n"
// A domain whose end is before its start is empty: no integer is a position of it.
let n = read<Int>()
domain V = 1..=n
print(V.count)
print(Int(V(1))) // <- here
