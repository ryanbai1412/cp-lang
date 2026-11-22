// Batch #13: domains made inside a fn, and parameters that size the arrays after them.
fn count(k: Int) -> Int:
    domain Local = 0..k
    var a: [Local]Int = 0
    for i in Local: a[i] = i.ord * 2
    sum(a[i] for i in Local)
fn ranks(xs: List<Int>) -> Int:
    let ys = compress(xs)
    var cnt: [ys.D]Int = 0
    for x in xs: cnt[ys.at(x)] += 1
    let twice = [ys.D] y => cnt[y] * 2
    domain Local = 0..ys.len
    let loc = [Local] i => i.ord
    sum(twice[y] for y in ys.D) + sum(loc)
fn valid(n: Int, seq: [0..n]Int) -> Bool:
    all(seq[i] <= seq[i + 1] for i in 0..n - 1)
fn columns(h: Int, w: Int, g: [0..h, 0..w]Int) -> [0..w]Int:
    [0..w] c => sum(g[r, c] for r in 0..h)
fn bump(k: Int, mut a: [0..=k]Int):
    for i in 0..=k: a[i] += i
let n: Int = read()
let a: [0..n]Int = read()
print(count(n), ranks([5, 1, 5, 9]), valid(n, a))
let zs = compress([a[i] for i in 0..n])
let top = [zs.D] y => zs[y] * 10
print(top)
var g: [0..2, 0..n]Int = 1
print(columns(2, n, g))
var b: [0..=n]Int = 0
bump(n, mut b)
print(b)
