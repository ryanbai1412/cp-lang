@!multitest
let (n, _k): (Int, Int) = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
let b: List<Int> = [read::<Int>() for _ in 0..n]
var segs: List<(Int, Int)> = [(min(a[i], b[i]), max(a[i], b[i])) for i in 0..n]
segs.sort()
let tot = sum(p.1 - p.0 for p in segs)
let min_loss = min(max(0, 2 * (segs[i + 1].0 - segs[i].1)) for i in 0..(n - 1))
print(tot + min_loss)
