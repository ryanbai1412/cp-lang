// NCCC 10 S3: a string s and q questions "t k": where does the k-th
// occurrence of t in s start (counting from 1, left to right), or -1.
//
// Sort the suffixes with SuffixArray. `find_all(t)` yields the starts of the
// suffixes that begin with t; the k-th smallest of them is the answer.
// original: corpus/submissions/4214399.cpp
use std.strings.{SuffixArray}

let s: Str = read()
let q: Int = read()
let sa = SuffixArray(s)

for _ in 0..q:
    let (t, k): (Str, Int) = read()
    let starts = sa.find_all(t).collect().sorted()
    print(if starts.len < k: -1 else: starts[k - 1] + 1)
