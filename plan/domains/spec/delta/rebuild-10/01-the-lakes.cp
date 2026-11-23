// CF 1829E The Lakes
// in: t; n m, then n rows of m depths 0..1000 | out: per test the largest 4-connected sum of positive cells
@!multitest
let n: 1..=1000 = read()
let m: 1..=1000 = read()
domain R = 0..n
domain C = 0..m
domain Cell = (R, C)
let depth = read<[Cell]0..=1000>()

var seen: [Cell]Bool = false
var best = 0
for s in Cell:
    if depth[s] == 0 or seen[s]: continue
    seen[s] = true
    var volume = 0
    var stack = [s]
    while let p = stack.pop():
        volume += depth[p]
        for q in p.adj4:
            if depth[q] > 0 and not seen[q]:
                seen[q] = true
                stack.push(q)
    best max= volume
print(best)
