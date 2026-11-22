// `if let` in a comprehension, `read()` of a `Bool` and of a `?T`, one row or column
// of a grid as a list, and `print(.., end: s)`.
let (n, m): (Int, Int) = read()
domain R = 0..n
domain C = 0..m
let g: [R, C]Int = read()
let ok: [R, C]Bool = read()
domain V = 1..=n
let boss: [V]?V = read(none: 0)
let (lo, hi): (?Int, ?Int) = read(none: -1)

print([b.ord for v in V if let b = boss[v]])
print(sum(b.ord for v in V if let b = boss[v] and b.ord > 0))
print(yes([x for x in [lo, hi] if let x = x].any(x => x > 6)), yes([x for x in [lo, hi] if let x = x].all(x => x > 6)))
print([a.ord + b.ord for v in V if let a = boss[v] and let b = boss[a]])

print(g[R(1), ..])
print(g[.., C(0)])
print([count(c for c in C if ok[r, c]) for r in R], sum(g[.., C(m - 1)]))
var h: [0..n, 1..=m]Int = 0
for r in R:
    for c in C: h[r.ord, c.ord + 1] = g[r, c] * 2
print(sorted(h[.., m]), h[0, ..])

for r in R:
    for c in C: print(g[r, c], end: if c.ord + 1 < m: "," else: "\n")
print(lo ?? 100, end: " ")
print([1, 2], sep: "-", end: "")
print()
