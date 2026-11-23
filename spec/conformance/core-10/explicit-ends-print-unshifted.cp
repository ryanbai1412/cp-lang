// expect: ok, output "2 2"
// rule: 10.2.2
// input: "2\n"
// A domain with explicit ends reads and prints its integers; `io.base` never shifts it (rebuild round 6 H-PRINT-POSITIONS).
with io(base: 1)
domain V = 1..=3
let v = read<V>()
print(v, Int(v))
