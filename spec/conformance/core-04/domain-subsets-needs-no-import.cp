// expect: ok, output "8 12\n1"
// `D.subsets` brings its std type in, though the program never names it.
domain L = [3]
let t = [L.subsets] s => s.len
var full = 0
for s in L.subsets:
    if s.len == 3: full += 1
print(t.len, t.sum())
print(full)
