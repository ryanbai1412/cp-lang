// expect: ok, output "1\n2\n3\n4"
// rule: 05.2.2
// A `var` parameter owns its argument, and halves passed on at their last use move in,
// even with a closure in the body that doesn't capture them.
fn split(var xs: List<Int>, lo: Int, hi: Int):
    if lo == hi:
        print(lo)
        return
    let mid = (lo + hi) / 2
    xs.push(mid)
    let (lows, highs) = xs.partition(x => x <= mid)
    split(lows, lo, mid)
    split(highs, mid + 1, hi)
split([1, 2, 3, 4], 1, 4)
