@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
var s: List<Int> = []
var cur = 0
for x in a:
    cur += x
    s.push(cur)
if s.min()! < 0:
    print(-1)
    return
print(s.inversions())
