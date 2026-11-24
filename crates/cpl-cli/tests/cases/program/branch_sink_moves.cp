// A local nothing after its `if` reads moves into what a branch stores it in: an optional,
// a field, a variable or a struct literal.
struct Holder:
    pub var xs: List<Int>
struct W:
    pub var n: Int
    pub vals: List<Int>
var best: ?List<Int> = None
var h = Holder([])
var last: List<Int> = []
for i in 0..4:
    let w = [i, i * i]
    if i % 2 == 0: best = w
    else:
        let v = [i]
        if i == 3: h.xs = v
    let u = [i + 1]
    if i > 1: last = u
    else: print(u.len)
    let vals = [i, i + 1]
    let n = vals.len
    if i == 2:
        var w = W(n, vals)
        w.n += w.vals[0]
        print(w.n, w.vals)
print(best!, h.xs, last)
