// n: 512
use std.strings.{SuffixArray}
let n: Int = read()
let ss = [Str([Char(Int('a') + (i * 7919 + i / 3) % 3) for i in 0..n]) for _ in 0..2]
let aa = "a".repeat(n)
let pat = "a".repeat(n / 8)
var acc = 0
for r in 0..256:
    acc += SuffixArray(ss[r % 2]).len // timed: strings.SuffixArray.new
let sa = SuffixArray(ss[0])
let sb = SuffixArray(aa)
let total = sa.distinct()
for i in 0..1 << 19:
    let j = i * 7 % (n - 4)
    let k = (i * 13 + 5) % n
    let lo = i % (n - 2)
    let hi = lo + 2 + i * 31 % (n - lo - 1)
    acc += sa.len // timed: strings.SuffixArray.len
    if sa.is_empty: acc += 1 // timed: strings.SuffixArray.is_empty
    acc += sa.lcp_between(j, k) // timed: strings.SuffixArray.lcp_between
    acc += sa.compare(j, n, k, n) // timed: strings.SuffixArray.compare
    acc += sa.lcp_of_ranks(lo..hi) // timed: strings.SuffixArray.lcp_of_ranks
    acc += sa.split(lo..hi) // timed: strings.SuffixArray.split
    acc += sa.distinct() // timed: strings.SuffixArray.distinct
    acc += sa.lcp_range(j, i % 4).len // timed: strings.SuffixArray.lcp_range
    if let r = sa.kth_distinct(i * 104729 % total): acc += r.start // timed: strings.SuffixArray.kth_distinct
for r in 0..2048:
    for _ in 0..4:
        acc += sb.rank_range(pat).len // timed: strings.SuffixArray.rank_range
        acc += sb.count(pat) // timed: strings.SuffixArray.count
    for k in sb.find_all(pat): acc += k // timed: strings.SuffixArray.find_all
print(acc)
