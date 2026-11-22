use std.seq.{bsearch_first, bsearch_last, ternary_max, ternary_min}

let lo = I128(-4_000_000_000) * I128(10_000_000_000)
let hi = I128(0)
print(bsearch_first(lo..=hi, x => x >= I128(-5))!)
print(bsearch_last(lo..=hi, x => x <= I128(-5))!)
print(bsearch_first(lo..=hi, x => x >= lo)!)
print(bsearch_last(lo..=hi, x => x <= hi)!)
let target = I128(-1_234_567_890) * I128(10_000_000_000)
print(ternary_min(lo..=hi, x => abs(x - target))!)
print(ternary_max(lo..=hi, x => -abs(x - target))!)
