// Every sort shape goes through one stable order: packed tuples, floats with
// a signed zero, boxed values, and keys with ties that keep their order.
var ps: List<(Int, Bool, Char)> = [(2, true, 'b'), (1, false, 'z'), (2, false, 'c'), (1, false, 'a'), (2, true, 'a')]
ps.sort()
print(ps.map(p => "{p.0}:{yes(p.1)}:{p.2}"))
let fs = [3.5, -0.0, 0.0, -2.0, 1.0e9, -1.0e-9]
print(fs.sorted())
var ws = ["pear", "fig", "apple", "kiwi", "date", "plum", "lime", "pea", "yam", "nut"]
ws.sort()
print(ws)
print(ws.sorted(by: w => w.len))
print(ws.sorted(by: w => desc(w.len)))
var big: List<(Int, Int)> = []
for i in 0..40:
    big.push(((i * 17) % 7, i))
print(big.sorted(by: p => p.0).map(p => p.1))
print(big.sorted(by: p => desc(p.0)).map(p => p.1))
