// Source: NOI '09 P2 (Little G the Poet), https://dmoj.ca/problem/noi09p2
// Problem: N <= 1e5 sentences (length <= 30), in order, set into lines joined by single
//   spaces; a line of length x costs |x - L|^P (L <= 3e6, P <= 10). Minimise the total: print
//   it and the lines, or "Too hard to arrange" if it exceeds 1e18; then 20 dashes. Multitest.
// Solution: lines are edges j -> i between cuts of a shortest path over cuts 0..=N; the cost
//   |len(j+1..i) - L|^P is convex in the length, so it is Monge and `monge_paths` (LARSCH)
//   finds every shortest path in O(N). Compared in F64; the total is printed exactly from
//   the lines of the path.
// original: corpus/submissions/5108899.cpp
// Bounds: n: 1..=1e5; l: 1..=3e6; p: 1..=10

use std.dp.{monge_paths}

@multitest
let (n, l, p): (1..=1e5, 1..=3e6, 1..=10) = read()
domain Sentence = 1..=n
let words: [Sentence]Str = read()
let ends = words.map(w => w.len + 1).scan(+)

fn line(j: Sentence.cuts, i: Sentence.cuts) -> Int = abs(ends[i] - ends[j] - 1 - l)
let best = monge_paths(List(Sentence.cuts), cost: (j, i) => F64(line(j, i)) ** F64(p))

if best.val.last! > 1e18:
    print("Too hard to arrange")
else:
    var cuts = [n]
    while cuts.last! > 0: cuts.push(best.prev[cuts.last!])
    cuts.reverse()
    print(sum(line(j, i) ** p for (j, i) in cuts.adjacent))
    for (j, i) in cuts.adjacent: print(words[j + 1..=i])
print("--------------------")
