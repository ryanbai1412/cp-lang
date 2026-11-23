// expect: ok, output "4\n6"
// rule: 02.8.1
// input: "2\n1 2\n3 4\n"
// Two counted `[n]T` arrays are both `[0..n]T`: a key of one indexes the other.
let n = read<1..=5>()
let a = read<[n]Int>()
let b = read<[n]Int>()
for i in a.keys: print(a[i] + b[i])
