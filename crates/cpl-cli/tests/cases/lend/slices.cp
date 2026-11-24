let n: 1..=9 = read()
domain V = 1..=n
var a: [V]Int = read()

// a[l..=r] lends a slice: the same positions, a window of them
let l = V(2)
let r = V(4)
a[l..=r].sort()
print(a)

// a let-bound slice keeps `a` lent until its last use; positions it returns are plain V values
let part = a[l..=r]
var best = l
for v in part.keys:
    if part[v] > part[best]: best = v
print(best, a[best], part.len, part.sum())

// the window is a value, the slice is a lend: fns return ranges, callers apply them
fn widest(lo: V, hi: V) -> Range<V>:
    lo..=hi
let w = widest(V.first!, V(3))
a[w].reverse()
print(a)

// a lent slice is a [V]Int to any top-level fn
fn total(xs: [V]Int) -> Int:
    xs.sum()
fn bump(mut xs: [V]Int, by: Int):
    for v in xs.keys: xs[v] += by
print(total(a[l..=r]), total(a))
bump(mut a[l..=r], 10)
print(a)

// two halves of one array at once: the law is `s.hi <= t.lo or t.hi <= s.lo`
fn mix(mut xs: [V]Int, mut ys: [V]Int):
    for v in xs.keys: xs[v] += 1
    for v in ys.keys: ys[v] += 2
let m = V(3)
mix(mut a[l.l..m.l], mut a[m.l..r.r])
print(a)

// an owned copy may be kept
var keep = a[l..=r].to_list()
keep.push(0)
print(keep)
