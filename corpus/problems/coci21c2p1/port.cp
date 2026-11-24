// Source: COCI 2021/2022 Contest 2 #1 (Kaučuk), https://dmoj.ca/problem/coci21c2p1
// limits: 1000 ms, 512 MB
// Problem: n <= 100 lines, each `section`, `subsection` or `subsubsection` and a title (<= 20
//   lowercase letters). Print each title after its number ("1", "1.2" or "1.2.1"): each level
//   counts from 1 and restarts under every new parent.
// Solution: a counter per depth. A command bumps its depth's counter, resets the deeper ones and
//   prints the counters down to its depth. O(n).
// original: corpus/submissions/4143397.cpp
// Bounds: n: 1..=100

let n: 1..=100 = read()
var number: [1..=3]Int = 0
for _ in 0..n:
    let command: Str = read()
    let title: Str = read()
    let depth = match command:
        "section" => 1
        "subsection" => 2
        _ => 3
    number[depth] += 1
    for d in depth + 1..=3: number[d] = 0
    print("{".".join([number[d].to_str() for d in 1..=depth])} {title}")
