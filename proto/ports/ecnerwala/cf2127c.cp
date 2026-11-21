@!multitest
let (n, _k): (Int, Int) = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()
let segs = sorted([(min(a[i], b[i]), max(a[i], b[i])) for i in 0..n])
let tot = sum(p.1 - p.0 for p in segs)
let min_loss = min(max(0, 2 * (q.0 - p.1)) for (p, q) in segs.adjacent)!
print(tot + min_loss)
