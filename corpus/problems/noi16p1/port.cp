// NOI 2016 Excellent Splits: count the ways to cut a block of the string
// into AABB with A and B not empty. For each of t strings.
//
// Every square AA belongs to a unique maximal repetition (run) `(l, r, p)`
// with `p | |A|`. Enumerating all runs via `lyndon_runs(s)` gives every
// square start/end interval deterministically in O(n log n); difference
// arrays accumulate how many squares start/end at each index.
// original: corpus/submissions/4797772.cpp
use std.strings.{lyndon_runs}

let t: Int = read()
for _ in 0..t:
    let s: Str = read()
    let n = s.len
    var starts = List.filled(n + 1, 0)
    var ends = List.filled(n + 1, 0)
    for run in lyndon_runs(s):
        var w = run.period
        while 2 * w <= run.r - run.l:
            let last = run.r - 2 * w
            starts[run.l] += 1
            starts[last + 1] -= 1
            ends[run.l + 2 * w - 1] += 1
            ends[last + 2 * w] -= 1
            w += run.period
    let (begun, ended) = (scan(starts, +), scan(ends, +))
    print(sum(ended[i] * begun[i + 1] for i in 0..n - 1))
