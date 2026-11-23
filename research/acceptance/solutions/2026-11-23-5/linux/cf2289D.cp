use std.seq.*

@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
let ps = scan(a, +)
let pre = [ps.prefix(i) for i in 1..=n]
print(if pre.min() < 0: -1 else: pre.inversions())
