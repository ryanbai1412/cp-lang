// Source: COCI 2021/2022 Contest 2 #1 (Kaučuk), https://dmoj.ca/problem/coci21c2p1
// limits: 1000 ms, 512 MB
// Problem: n <= 100 lines, each `section`, `subsection` or `subsubsection` and a title (<= 20
//   lowercase letters). Print each title after its number ("1", "1.2" or "1.2.1"): each level
//   counts from 1 and restarts under every new parent.
// Solution: a counter per depth, read off the command's length. A command bumps its depth's
//   counter, resets the deeper ones and prints the counters down to its depth. O(n).
// Runs on proto/cp.py lib-run today: proto/cpc has no string interpolation.
//   See ../coci21c2p1.cp.
// original: corpus/submissions/4143397.cpp
let n: Int = read()
var at = [0, 0, 0, 0]
for _ in 0..n:
    let kind: Str = read()
    let title: Str = read()
    let depth = 1 + (kind.len - 7) / 3
    at[depth] += 1
    for d in depth + 1..=3: at[d] = 0
    if depth == 1: print("{at[1]} {title}")
    elif depth == 2: print("{at[1]}.{at[2]} {title}")
    else: print("{at[1]}.{at[2]}.{at[3]} {title}")
