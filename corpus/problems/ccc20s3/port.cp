// Source: CCC 2020 S3, Searching for Strings, https://dmoj.ca/problem/ccc20s3
// Problem: lowercase needle N and haystack H, |N|, |H| <= 2e5. Count the distinct permutations
//   of N that occur as a substring of H.
// Solution: slide a window of length |N| over H keeping letter-count differences and how many are
//   nonzero; when all match, the window is a permutation, and its rolling hash goes into a set.
//   The answer is the set's size: O(|H|) expected, up to hash collisions.
// original: corpus/submissions/4323643.cpp
use std.strings.{HashVal}

domain Letter = 'a'..='z'
let needle: Str = read()
let hay: Str = read()
let k = needle.len

// surplus[c]: the window's count of c minus the needle's; off: how many letters disagree.
var surplus: [Letter]Int = 0
var off = 0
fn shift(c: Char, by: Int):
    off -= Int(surplus[Letter(c)] != 0)
    surplus[Letter(c)] += by
    off += Int(surplus[Letter(c)] != 0)
for c in needle: shift(c, -1)

let h = RollingHash(hay)
var found = Set<HashVal>()
for i in 0..hay.len:
    shift(hay[i], 1)
    if i >= k: shift(hay[i - k], -1)
    if i + 1 >= k and off == 0: found.insert(h.get(i + 1 - k..i + 1))
print(found.len)
