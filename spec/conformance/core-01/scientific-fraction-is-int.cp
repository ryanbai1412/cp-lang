// expect: ok, output "7"
// rule: 01.4.1 01.4.2
// input: "3 4\n"
// `1.5e5` is an exact integer, so it is an `Int` literal, in a type argument too (rebuild 8, H-SCI-FRACTION).
let (n, q) = read<(1..=1.5e5, 1..=2.5e5)>()
print(n + q)
