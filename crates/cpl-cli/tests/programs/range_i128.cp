use std.seq.{bsearch_first}

fn take_r(r: Range<I128>) -> I128 = r.lo + r.hi

let big: I128 = I128(1000000000000000000) * I128(100)
let r: Range<I128> = big..=big + I128(3)
print(take_r(r), r.lo, r.hi, r.len(), r.first!, r.last!)
var acc: I128 = 0
for x in r:
    acc += x
print(acc, big + I128(2) in r, big + I128(5) in r)

let ans = bsearch_first(big..=big + I128(1000), x => x >= big + I128(777))
print(ans!)
