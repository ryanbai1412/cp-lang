// Tuned: a plain lazy segment tree with add and assign tags over min, max and sum, as the
// accepted C++'s, where the port's BeatsSegTree also keeps the second extremes and their counts
// that only chmin/chmax need. A division that moves a node's min and max by the same amount is an
// add on it (an assign when they meet), so it recurses only where the spread is still wide.
//
// Source: ETSR '23 R1 P4 (LOJ 6029, Market), https://dmoj.ca/problem/etsr23r1p4
// limits: 2000 ms, 256 MB
// Problem: 1 <= n, q <= 2e5 and an array a_1..a_n of 32-bit ints. Queries "1 l r c" add
//   -1e4 <= c <= 1e4 to a_l..=a_r, "2 l r d" replaces each by floor(a_i / d) for 1 <= d <= 1e9,
//   "3 l r" prints the minimum and "4 l r" the sum of a_l..=a_r.
// Solution: segment tree beats. Floor division is an add on a node once its min and max move by
//   the same amount, which happens after O(log) divisions of a node's spread, so BeatsSegTree's
//   chdiv is amortised O((n + q) log n log V) with add, min and sum as usual.
// original: corpus/submissions/5495163.cpp
// Bounds: n, q: 1..=2e5; a: -2147483648..=2147483647; op: 1..=4; l, r: 1..=n

const NONE = Int.MIN

struct Market:
    mn: List<Int>
    mx: List<Int>
    sum: List<Int>
    added: List<Int>  // an add the children have not seen
    set: List<Int>  // a value the children have not taken, NONE when there is none

    fn put_add(mut self, u: Int, len: Int, x: Int):
        mn[u] += x
        mx[u] += x
        sum[u] += x * len
        if set[u] != NONE: set[u] += x else: added[u] += x

    fn put_set(mut self, u: Int, len: Int, x: Int):
        mn[u] = x
        mx[u] = x
        sum[u] = x * len
        set[u] = x
        added[u] = 0

    fn push(mut self, u: Int, nl: Int, nr: Int):
        let mid = (nl + nr) / 2
        if set[u] != NONE:
            self.put_set(2 * u, mid - nl, set[u])
            self.put_set(2 * u + 1, nr - mid, set[u])
            set[u] = NONE
        if added[u] != 0:
            self.put_add(2 * u, mid - nl, added[u])
            self.put_add(2 * u + 1, nr - mid, added[u])
            added[u] = 0

    fn pull(mut self, u: Int):
        mn[u] = min(mn[2 * u], mn[2 * u + 1])
        mx[u] = max(mx[2 * u], mx[2 * u + 1])
        sum[u] = sum[2 * u] + sum[2 * u + 1]

    fn build(mut self, u: Int, nl: Int, nr: Int, a: List<Int>):
        if nr - nl == 1:
            self.put_set(u, 1, a[nl])
            set[u] = NONE
            return
        let mid = (nl + nr) / 2
        self.build(2 * u, nl, mid, a)
        self.build(2 * u + 1, mid, nr, a)
        self.pull(u)

    fn add(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int):
        if qr <= nl or nr <= ql: return
        if ql <= nl and nr <= qr:
            self.put_add(u, nr - nl, x)
            return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.add(2 * u, nl, mid, ql, qr, x)
        self.add(2 * u + 1, mid, nr, ql, qr, x)
        self.pull(u)

    fn div(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, d: Int):
        if qr <= nl or nr <= ql: return
        if ql <= nl and nr <= qr:
            let (lo, hi) = (mn[u] / d, mx[u] / d)
            if lo == hi:
                self.put_set(u, nr - nl, lo)
                return
            if lo - mn[u] == hi - mx[u]:
                self.put_add(u, nr - nl, lo - mn[u])
                return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.div(2 * u, nl, mid, ql, qr, d)
        self.div(2 * u + 1, mid, nr, ql, qr, d)
        self.pull(u)

    fn low(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return Int.MAX
        if ql <= nl and nr <= qr: return mn[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        min(self.low(2 * u, nl, mid, ql, qr), self.low(2 * u + 1, mid, nr, ql, qr))

    fn total(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return 0
        if ql <= nl and nr <= qr: return sum[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.total(2 * u, nl, mid, ql, qr) + self.total(2 * u + 1, mid, nr, ql, qr)

let (n, q): (1..=2e5, 1..=2e5) = read()
let a: [n]-2147483648..=2147483647 = read()
var t = Market(
    List.filled(4 * n, 0), List.filled(4 * n, 0), List.filled(4 * n, 0), List.filled(4 * n, 0),
    List.filled(4 * n, NONE),
)
t.build(1, 0, n, [Int(v) for v in a])
for _ in 0..q:
    let (op, l, r): (1..=4, 1..=n, 1..=n) = read()
    if op == 1: t.add(1, 0, n, l - 1, r, read<-1e4..=1e4>())
    elif op == 2: t.div(1, 0, n, l - 1, r, read<1..=1e9>())
    elif op == 3: print(t.low(1, 0, n, l - 1, r))
    else: print(t.total(1, 0, n, l - 1, r))
