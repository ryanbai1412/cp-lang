// expect: ok, output "2 1 3 YES"
// rule: 02.1.3
// `Some` named as a value is the constructor `x => Some(x)` (std/pair.cp documents `.map(Some)`).
let xs = [1, 2]
let ys = xs.map(Some)
let a: [0..2]Int = [0..2] i => 7 + i
let table: [0..2]?Int = a.map(Some)
print(ys.len, ys[0] ?? 0, [3].map(Some)[0]!, yes(table[1] == Some(8)))
