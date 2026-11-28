// expect: ok, output "1\n0\n2"
// input: "3\n6 5 7\n"
// rule: R04.3.4
// A key of an array over an anonymous range is a plain `Int` in an `Int` cell.
let n: 1..=10 = read()
let a: [0..n]5..=7 = read()
var at: [5..=7]?Int = None
for (i, x) in a.items: at[x] = i
for v in 5..=7: print(at[v]!)
