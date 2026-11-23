// expect: ok, output "5 4 2"
// rule: 02.8.1 04.3.3
// Arrays over an anonymous range (`[n]T`, `[0..=n]T`) are indexed by plain `Int`, checked.
let a: [3]Int = [4, 5, 6]
var best: [0..=10]Int = 0
let j = 1
best[j + 1] = a[j]
best[j] += 2
print(best[2], a[j - 1], best[1])
