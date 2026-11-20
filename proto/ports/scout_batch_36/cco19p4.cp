// CCO '19 P4 — Card Scoring (DMOJ: cco19p4)
// original: corpus/submissions/3617807.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Partition a sequence of n card ranks (1..=n) by repeatedly picking cards of
// the same rank; a group of size d scores d^(k/2) for k in {2, 3, 4}.
// Maximize total score.
//
// For k in {3, 4}, d^(k/2) is convex, so two curves f(c) = score(m - c) + b
// cross at most once on c <= min(m1, m2). A per-rank Li Chao tree answers
// queries in O(log cnt[rank]). Under v2 T9+T10, F64 forbids `-inf` and `?F64`
// has no `<`, so curve comparison uses an explicit `beats(?F64, ?F64)` helper.
let (k, n): (2..=4, 1..=1e6) = read()
if k == 2:
    print("{F64(n):.6f}")
    return

domain Card = 0..n
domain Rank = 1..=n
let a: [Card]Rank = read()

let score: [0..=n]F64 = [0..=n] d =>
    if k == 4: F64(d) * F64(d)
    else:
        let s = sqrt(F64(d))
        s * s * s

struct Curve:
    m: Int
    b: F64
    fn eval(self, x: Int) -> ?F64:
        if x > self.m: none else: score[self.m - x] + self.b

fn beats(a: ?F64, b: ?F64) -> Bool:
    if let x = a:
        if let y = b: x > y else: true
    else: false

struct LiChao:
    size: Int
    tree: List<?Curve>

    fn insert(mut self, mut c: Curve):
        fn go(o: Int, l: Int, r: Int, mut c: Curve):
            if let cur = self.tree[o]:
                let mid = (l + r) / 2
                let lv = beats(c.eval(l), cur.eval(l))
                let mv = beats(c.eval(mid), cur.eval(mid))
                if mv: (self.tree[o], c) = (c, cur)
                if l < r:
                    if lv != mv or c.eval(mid) == none:
                        go(o * 2, l, mid, mut c)
                    else:
                        go(o * 2 + 1, mid + 1, r, mut c)
            else:
                self.tree[o] = c
        go(1, 0, self.size - 1, mut c)

    fn query(self, x: Int) -> ?F64:
        var (o, l, r) = (1, 0, self.size - 1)
        var best: ?F64 = none
        while true:
            if let c = self.tree[o]: best max= c.eval(x)
            if l == r: return best
            let mid = (l + r) / 2
            if x <= mid: (o, r) = (o * 2, mid)
            else: (o, l) = (o * 2 + 1, mid + 1)

var cnt: [Rank]Int = 0
for i in Card: cnt[a[i]] += 1

var hulls: [Rank]LiChao = [Rank] r =>
    let sz = cnt[r] + 1
    LiChao(sz, [none for _ in 0..4 * sz])

var dp: [0..=n]F64 = 0
for i in Card.rev:
    let r = a[i]
    cnt[r] -= 1
    hulls[r].insert(Curve(cnt[r] + 1, dp[i.to_idx + 1]))
    dp[i.to_idx] = hulls[r].query(cnt[r])!

print("{dp[0]:.6f}")
