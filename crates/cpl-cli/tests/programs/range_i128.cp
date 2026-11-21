use std.seq.{bsearch_first, bsearch_last}

fn take_r(r: Range<I128>) -> I128 = r.lo + r.hi

let big: I128 = I128(1000000000000000000) * I128(100)
let r: Range<I128> = big..=big + I128(3)
print(take_r(r), r.lo, r.hi, r.len, r.first!, r.last!)
var acc: I128 = 0
for x in r:
    acc += x
print(acc, big + I128(2) in r, big + I128(5) in r)

let ans = bsearch_first(big..=big + I128(1000), x => x >= big + I128(777))
print(ans!)

let wide_lo: I128 = I128(-4) * I128(10000000000) * I128(1000000000)
let wide_first = bsearch_first(wide_lo..=I128(0), x => x >= I128(-5))
let wide_last = bsearch_last(wide_lo..=I128(0), x => x <= I128(-5))
print(wide_first!, wide_last!)
