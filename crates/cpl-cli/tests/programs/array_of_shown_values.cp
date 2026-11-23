// An array prints each cell through its element type's `impl Show`, as a list does (noip09p4's `[Line]Subset`).
struct Frac: p: Int, q: Int
impl Show for Frac:
    fn show(f) = "{f.p}/{f.q}"
domain L = 0..3
var a: [L]Frac = Frac(0, 1)
a[L(1)] = Frac(1, 2)
print(a)
var g: [L, L]Frac = Frac(1, 1)
g[L(2), L(0)] = Frac(2, 3)
print(g)
