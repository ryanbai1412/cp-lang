@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
let sc = a.scan(+)
let s = [sc.cut(c) for c in 1..=n]
if s.any(x => x < 0):
    print(-1)
    return
print(s.inversions())
