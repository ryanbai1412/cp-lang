fn mysort<S: IndexedMut, T, K: Ord>(mut self: S; by: T -> K = x => x):
    let n = self.len
    let ks = self.keys
    var buf: List<T> = [self[ks.nth(i)] for i in 0..n / 2]
    my_sort_run(mut self, ks, 0, n, mut buf, by)

fn my_sort_run<S: IndexedMut, D, T, K: Ord>(mut a: S, ks: Range<D>, lo: Int, hi: Int, mut buf: List<T>, by: T -> K):
    if hi - lo <= 8:
        for i in lo + 1..hi:
            let x = a[ks.nth(i)]
            let kx = by(x)
            var j = i
            while j > lo and kx < by(a[ks.nth(j - 1)]):
                a[ks.nth(j)] = a[ks.nth(j - 1)]
                j -= 1
            a[ks.nth(j)] = x
        return
    let mid = lo + (hi - lo) / 2
    my_sort_run(mut a, ks, lo, mid, mut buf, by)
    my_sort_run(mut a, ks, mid, hi, mut buf, by)
    if not (by(a[ks.nth(mid)]) < by(a[ks.nth(mid - 1)])): return
    let m = mid - lo
    for t in 0..m: buf[t] = a[ks.nth(lo + t)]
    var (i, j, k) = (0, mid, lo)
    while i < m and j < hi:
        if by(a[ks.nth(j)]) < by(buf[i]):
            a[ks.nth(k)] = a[ks.nth(j)]
            j += 1
        else:
            a[ks.nth(k)] = buf[i]
            i += 1
        k += 1
    while i < m:
        a[ks.nth(k)] = buf[i]
        i += 1
        k += 1

let n = 1_000_000
var acc = 0
for r in 0..4:
    var xs = [(cost: (i * 7_919_993 + r) % 1_000_000_007, v: i, p: i / 2) for i in 0..n]
    mysort(mut xs, by: j => desc(j.cost))
    acc += xs[0].v + xs[n - 1].v + xs[n / 2].p
print(acc)
