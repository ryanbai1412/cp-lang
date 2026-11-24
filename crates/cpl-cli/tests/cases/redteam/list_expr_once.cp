// A builtin given a list expression evaluates it once: `compress([...])` used to write it twice, for `begin()` and `end()`.
let pts = [(5, 1), (3, 2), (5, 3), (9, 4)]
let (X, col) = compress([p.0 for p in pts])
print(X.len, col)
print(yes([p.0 for p in pts].contains(3)), sorted([p.0 for p in pts]))
print(sorted([p.0 for p in pts], by: x => -x))
