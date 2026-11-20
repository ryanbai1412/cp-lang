// BOI 2019 Necklace: find the longest block of s that is a block of t after
// turning it round (a rotation) and maybe flipping it over (a reversal).
// Print its length, then where it starts in s and in t.
//
// A rotation of XY is YX. Cut s at every place: X is what ends at the cut
// and Y what starts there. In t they meet at some j the other way round, Y
// ending at j and X starting there. `overlaps` (the failure function run
// over a text) gives the longest Y for every j, and on the reversed strings
// the longest X. The flip is the same search with t reversed.
// original: corpus/submissions/4456167.cpp
use std.strings.{prefix_function}

/// `k[j]` is the longest prefix of `pat` that ends at `text[..j]`, for j in 0..=text.len.
fn overlaps(pat: List<Char>, text: List<Char>) -> List<Int>:
    // '#' is in neither string, so no border crosses it.
    let border = prefix_function(pat + ['#'] + text)
    [0] + border[pat.len + 1..border.len]

/// (length, start in s, start in t) of the longest block of s that is a rotation of a block of t.
fn longest(s: List<Char>, t: List<Char>) -> (Int, Int, Int):
    let (n, m) = (s.len, t.len)
    var best = (0, 0, 0)
    for cut in 0..=n:
        let y = overlaps(s[cut..n], t)
        let x = overlaps(s[0..cut].rev, t.rev)
        for j in 0..=m:
            best = max(best, (x[m - j] + y[j], cut - x[m - j], j - y[j]))
    best

let s: Str = read()
let t: Str = read()
let (a, b) = ([c for c in s], [c for c in t])
let (len, in_s, in_t) = longest(a, b)
let (flipped, flipped_s, flipped_t) = longest(a, b.rev)
if len >= flipped:
    print(len)
    print(in_s, in_t)
else:
    print(flipped)
    print(flipped_s, b.len - flipped_t - flipped)
