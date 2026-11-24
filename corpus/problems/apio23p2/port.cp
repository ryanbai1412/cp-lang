// Source: APIO 2023 P2 (Sequence), https://dmoj.ca/problem/apio23p2
// Problem: A of length N <= 5e5 with 1 <= A[i] <= N. Over all subarrays and each median x of it
//   (both middle elements if the length is even), print the max number of occurrences of x in it.
// Solution: v is a median of a subarray holding c copies iff |#greater - #smaller| <= c. Take
//   values in increasing order over a segment tree of +1/-1/0 steps (greater/smaller/v) that
//   folds to a walk's sum and lowest and highest point. The copies of v cut the prefixes into
//   blocks 0..=c, each reaching a range of balances; blocks i < j work iff those ranges are at
//   most j - i apart, i.e. lo_j - j <= hi_i - i and lo_i + i <= hi_j + j. Sweep i by hi_i - i,
//   adding each j once lo_j - j is reached to a max tree keyed by hi_j + j, and ask for the
//   latest j keyed at least lo_i + i. Values with no more copies than the answer are skipped.
//   O(N log N).
// original: corpus/submissions/5593302.cpp
// Bounds: n: 1..=5e5; a: 1..=n
let n: 1..=5e5 = read()
domain I = 0..n
domain Val = 1..=n
let a: [I]Val = read()
var at: [Val]List<I> = []
for i in I:
    at[a[i]].push(i)

struct Walk(sum: Int, lo: Int, hi: Int)
fn step(s: Int) -> Walk = Walk(s, min(0, s), max(0, s))
fn then(x: Walk, y: Walk) -> Walk = Walk(x.sum + y.sum, min(x.lo, x.sum + y.lo), max(x.hi, x.sum + y.hi))

// Pre(t) is the step into prefix t; Pre(0) starts the walk.
domain Pre = 0..=n
var walk = SegTree(Monoid(step(0), then), [Pre] t => step(if t == Pre.first!: 0 else: 1))

fn longest(spans: List<(lo: Int, hi: Int)>) -> Int:
    let ks = 0..spans.len
    let (domain D, tops) = compress(ks.map(j => spans[j].hi + j))
    var latest = SegTree<D, ?Int>(max)
    let js = ks.sorted(by: j => spans[j].lo - j)
    var added = 0
    var far = 0
    for i in ks.sorted(by: i => spans[i].hi - i):
        while added < js.len and spans[js[added]].lo - js[added] <= spans[i].hi - i:
            let j = js[added]
            latest.update(tops.index_of(spans[j].hi + j), x => max(x, Some(j)))
            added += 1
        if let d = tops.first_ge(spans[i].lo + i):
            if let j = latest.fold(d..):
                far max= j - i
    far

var best = 0
for v in Val:
    for p in at[v]:
        walk.set(Pre(Int(p) + 1), step(0))
    if at[v].len > best:
        var spans: List<(lo: Int, hi: Int)> = []
        var (from, base) = (0, 0)
        for to in [Int(p) for p in at[v]] + [n]:
            let w = walk.fold(Pre(from)..=Pre(to))
            spans.push((lo: base + w.lo, hi: base + w.hi))
            base += w.sum
            from = to + 1
        best max= longest(spans)
    for p in at[v]:
        walk.set(Pre(Int(p) + 1), step(-1))
print(best)
