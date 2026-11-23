// expect: ok, output "4 4"
// An array over a literal range is keyed by plain `Int` (round 24, H09).
let n = 4
let a: [0..n]Int = [0..n] i => i * i
print(a[2], a.len)
