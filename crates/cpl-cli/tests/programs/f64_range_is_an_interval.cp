// `F64` ends make an interval: it has ends and `in`, and a real search
// written over it reads only those (lib/search-dp.md, decision 419).
fn halve(i: Range<F64>, ok: F64 -> Bool, steps: Int) -> F64:
    var (lo, hi) = (i.start, i.end)
    for _ in 0..steps:
        let mid = (lo + hi) / 2.0
        if ok(mid):
            hi = mid
        else:
            lo = mid
    hi

let i = 0.0..10.0
let w: Range<F64> = -1.5..2.5
print(i.start, i.end, w.end - w.start)
print(yes(2.5 in i), yes(10.0 in i), yes(-1.0 in w), yes(3.0 in 0.0..2.0))
print(halve(i, x => x * x >= 2.0, steps: 60))
