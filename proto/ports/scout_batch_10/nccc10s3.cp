// NCCC 10 S3: a string s and q questions "t k": where does the k-th
// occurrence of t in s start (counting from 1, left to right), or -1.
//
// Sort the suffixes. Those that start with t are next to each other, so two
// binary searches find them all, and the k-th smallest start among them is
// the answer. The original builds the order with hashes and sorts that run
// for every question, with a special case for a string of one letter, where
// the runs are long; so does this port, without the special case.
// original: corpus/submissions/4214399.cpp
use std.strings.{suffix_array}

let s: Str = read()
let q: Int = read()
let n = s.len
let order = suffix_array(s)

/// How many suffixes, in sorted order, come before the first that fails `ok`.
fn leading(ok: Str -> Bool) -> Int:
    var (lo, hi) = (0, n)
    while lo < hi:
        let mid = (lo + hi) / 2
        if ok(s[order[mid]..n]): lo = mid + 1
        else: hi = mid
    lo

for _ in 0..q:
    let (t, k): (Str, Int) = read()
    // A suffix cut to t's length is below, equal to or above t.
    let below = leading(suffix => suffix[0..min(t.len, suffix.len)] < t)
    let upto = leading(suffix => suffix[0..min(t.len, suffix.len)] <= t)
    if upto - below < k: print(-1)
    else: print(sorted(order[below..upto])[k - 1] + 1)
