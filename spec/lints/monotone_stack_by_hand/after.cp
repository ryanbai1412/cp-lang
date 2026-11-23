let n: 1..=200000 = read()
let a: [0..n]Int = read()
let next = a.next_smaller(by: x => -x)
print([next[i] ?? -1 for i in 0..n])
