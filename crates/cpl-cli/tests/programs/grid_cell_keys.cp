// A grid's keys are its cells and one table name over two axes is the cell tuple (R04.2.10, R04.2.11).
domain R = 0..2
domain C = 0..3
let t = [R, C] p => Int(p.0) * 10 + Int(p.1)
print(t[1, 2])
let g = [R, C] (r, c) => Int(r) + Int(c)
var s = 0
for p in g.keys:
    s += g[p] * (Int(p.0) + 1)
print(s, g.keys.count(p => g[p] == 2))
