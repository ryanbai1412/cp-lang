// NOI 2016 Excellent Splits: count the ways to cut a block of the string
// into AABB with A and B not empty. For each of t strings.
//
// A square AA whose half has length w covers exactly one position q that is
// a multiple of w in its first half, and q + w in its second. It starts at p
// exactly when the letters before q and before q + w agree back to p, and
// the letters from q and from q + w agree up to p + w: a run of starts for
// each q, n / w of them for each w. An AABB is a square ending at i followed
// by a square starting at i + 1.
// original: corpus/submissions/4797772.cpp
use std.strings.{RollingHash}
use std.seq.{scan}

let t: Int = read()
for _ in 0..t:
    let s: Str = read()
    let n = s.len
    let hash = RollingHash.new(s)
    // How the number of squares that start at p, and that end at p, changes from p - 1 to p.
    var starts = List.filled(n + 1, 0)
    var ends = List.filled(n + 1, 0)
    for w in 1..=n / 2:
        for k in 0..(n - 1) / w:
            let q = k * w
            let first = max(q - hash.lcs(q, q + w), q - w + 1)
            let last = min(q, q - w + hash.lcp(q, q + w))
            if first > last: continue
            starts[first] += 1
            starts[last + 1] -= 1
            ends[first + 2 * w - 1] += 1
            ends[last + 2 * w] -= 1
    let (begun, ended) = (scan(starts, +), scan(ends, +))
    print(sum(ended[i] * begun[i + 1] for i in 0..n - 1))
