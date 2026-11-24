fn mysort<S: IndexedMut, T, K: Ord>(mut self: S; by: T -> K = x => x):
    let n = self.len
    let ks = self.keys
    var buf: List<T> = []
    buf.reserve(n / 2)
    for i in 0..n / 2: buf.push(self[ks.nth(i)])
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

struct Dir(d: Pt)

impl Ord for Dir:
    fn lt(a, b) = cross(a.d, b.d) > 0

impl Eq for Dir:
    fn eq(a, b) = cross(a.d, b.d) == 0

let n = 2500
var s = 88_172_645
fn next() -> Int:
    s = (s * 1_103_515_245 + 12_345) % 2_147_483_648
    s % 1_000_000_000
let pts = [Pt(next(), next()) for _ in 0..n].sorted(by: p => (p.x, p.y))
var turns: List<(d: Pt, a: Int, b: Int)> = []
turns.reserve(n * (n - 1) / 2)
for i in 0..n:
    for j in i + 1..n: turns.push((pts[j] - pts[i], i, j))
mysort(mut turns, by: t => Dir(t.d))
var acc = 0
for t in turns: acc = (acc * 31 + t.a * n + t.b) % 1_000_000_007
print(acc)
