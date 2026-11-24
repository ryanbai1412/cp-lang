// Source: BOI 2019 Day 2 P2 (Necklace), https://dmoj.ca/problem/btoi19p5
// limits: 1000 ms, 4 MB
// Problem: strings s and t of length n <= 3000. Find the longest block of s that equals a
//   block of t after a rotation and maybe a reversal; print its length and its starts in s
//   and in t. Full marks need 4 MB of memory.
// Solution: a rotation of XY is YX. Cut s at every place: X ends at the cut, Y starts there; in
//   t they meet at some j, Y ending at j and X starting there. KMP (`prefix_matches`) gives
//   the longest Y for every j, and on the reversed strings the longest X. The flip is the same
//   search on t reversed: O(n^2) time, O(n) memory.
// original: corpus/submissions/4456167.cpp
use std.strings.{prefix_matches}

let s: Str<1..=3000> = read()
let t: Str<1..=3000> = read()
type Block = (len: Int, in_s: Int, in_t: Int)

/// The longest block of s that is a rotation of a block of `u`, starts 0-based.
fn longest<T: Indexed>(u: T) -> Block:
    var best: Block = (0, 0, 0)
    for cut in 0..=s.len:
        let y = prefix_matches(u, s[cut..s.len])
        let x = prefix_matches(List(u.rev), List(s[0..cut].rev))
        for j in 0..=u.len:
            let yv = if j == 0: 0 else: y[j - 1]
            let xv = if j == u.len: 0 else: x[u.len - j - 1]
            best max= (xv + yv, cut - xv, j - yv)
    best

let straight = longest(t)
let back = longest(List(t.rev))
let flipped: Block = (back.len, back.in_s, t.len - back.in_t - back.len)
let answer = if straight.len >= flipped.len: straight else: flipped
print(answer.len)
print(answer.in_s, answer.in_t)
