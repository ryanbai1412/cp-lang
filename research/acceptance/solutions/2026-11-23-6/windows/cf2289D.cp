@!multitest
let n: 2..=2e5 = read()
let a: [0..n]-1e9..=1e9 = read()
let pre = a.scan(+)
let ps = [pre[i] for i in 1..=n]
if ps.any(x => x < 0):
    print(-1)
else:
    print(ps.inversions())

@alt(program)
fn slow():
    var p: List<Int> = []
    var s = 0
    for x in a:
        s += x
        p.push(s)
    if p.any(x => x < 0):
        print(-1)
        return
    var inv = 0
    for i in 0..n:
        for j in i + 1..n:
            if p[i] > p[j]: inv += 1
    print(inv)
