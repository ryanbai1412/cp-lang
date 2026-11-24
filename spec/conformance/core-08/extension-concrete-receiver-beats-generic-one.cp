// expect: ok, output "3 2.5"
// rule: 08.1.4
// A fn whose `self` is `Range<F64>` is more specific than std's `bsearch<D>(self: Range<D>)`: it takes the real range, and the integer range still calls std's.
fn bsearch(self: Range<F64>, ok: F64 -> Bool) -> F64:
    var (lo, hi) = (self.start, self.end)
    for _ in 0..60:
        let mid = (lo + hi) / 2.0
        if ok(mid): hi = mid
        else: lo = mid
    hi
print((0..10).bsearch(x => x >= 3), "{(0.0..8.0).bsearch(x => x >= 2.5):.1}")
