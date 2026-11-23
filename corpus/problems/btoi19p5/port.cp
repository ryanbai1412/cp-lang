// BOI 2019 Necklace: find the longest block of s that is a block of t after
// turning it round (a rotation) and maybe flipping it over (a reversal).
// Print its length, then where it starts in s and in t.
//
// A rotation of XY is YX. Cut s at every place: X is what ends at the cut
// and Y what starts there. In t they meet at some j the other way round, Y
// ending at j and X starting there. `prefix_matches` (KMP run
// over a text) gives the longest Y for every j, and on the reversed strings
// the longest X. The flip is the same search with t reversed.
// original: corpus/submissions/4456167.cpp
use std.strings.{prefix_matches}

/// (length, start in s, start in t) of the longest block of s that is a rotation of a block of t.
fn longest<T: Indexed>(s: Str, t: T) -> (Int, Int, Int):
    let (n, m) = (s.len, t.len)
    var best = (0, 0, 0)
    for cut in 0..=n:
        let y = prefix_matches(t, s[cut..n])
        let x = prefix_matches(t.rev, s[0..cut].rev)
        for j in 0..=m:
            let yv = if j == 0: 0 else: y[j - 1]
            let xv = if j == m: 0 else: x[m - j - 1]
            best max= (xv + yv, cut - xv, j - yv)
    best

let s: Str = read()
let t: Str = read()
let (len, in_s, in_t) = longest(s, t)
let (flipped, flipped_s, flipped_t) = longest(s, t.rev)
if len >= flipped:
    print(len)
    print(in_s, in_t)
else:
    print(flipped)
    print(flipped_s, t.len - flipped_t - flipped)
