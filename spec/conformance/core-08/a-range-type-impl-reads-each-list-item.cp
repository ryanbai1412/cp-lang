// expect: ok, output "3 6"
// rule: 08.4.5
// input: "3\n1 3 2\n"
// `impl Read for _..=_` is the impl for every range type, so `read<List<1..=3>>()`
// reads each item as a `1..=3`.
let xs = read<List<1..=3>>()
print(xs.len, Int(xs[0]) + Int(xs[1]) + Int(xs[2]))
