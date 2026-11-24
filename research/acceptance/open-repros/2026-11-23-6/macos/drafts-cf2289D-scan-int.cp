@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
let pre = a.scan(+)
let s = [pre[i] for i in 0..n]
print(if s.min()! < 0: -1 else: s.inversions())
