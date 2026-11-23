// here: 5
// no fix: a key of one row, written as a two-argument comparator.
let n: 1..=5 = read()
var rows: [0..n](name: Str, score: 0..=100) = read()
rows.sort(by: (a, b) => (-b, a))
print(rows)
