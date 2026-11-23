// Source: BOI 2019 Day 2 P2 (Necklace), https://dmoj.ca/problem/btoi19p5
// Problem: strings s and t of length n <= 3000. Find the longest block of s that equals a
//   block of t after a rotation and maybe a reversal; print its length and its starts in s
//   and in t. Full marks need 4 MB of memory.
// Solution: a rotation of XY is YX. Cut s at every place: X ends at the cut, Y starts there; in
//   t they meet at some j, Y ending at j and X starting there. KMP (`prefix_matches`) gives
//   the longest Y for every j, and on the reversed strings the longest X. The flip is the same
//   search on t reversed: O(n^2) time, O(n) memory.
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
