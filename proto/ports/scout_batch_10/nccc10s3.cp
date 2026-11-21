// NCCC 10 S3: a string s and q questions "t k": where does the k-th
// occurrence of t in s start (counting from 1, left to right), or -1.
//
// Sort the suffixes with SuffixArray. Those starting with t form a contiguous
// rank range returned by `matches(t)`; the k-th smallest start in that range
// is the answer.
// original: corpus/submissions/4214399.cpp
use std.strings.{SuffixArray}

let s: Str = read()
let q: Int = read()
let sa = SuffixArray.new(s)

for _ in 0..q:
    let (t, k): (Str, Int) = read()
    let r = sa.matches(t)
    if r.hi - r.lo < k: print(-1)
    else: print(sa.sa[r.lo..r.hi].sorted()[k - 1] + 1)
