// NOI 2009 Poet (CP Lang 0.2.2): convex 1D DP with monotone decision queue and
// binary search for transition crossover points.
// original: corpus/submissions/5108899.cpp
use std.seq.{bsearch_first}

let dashes = "--------------------"

@multitest
let (n, l, p): (Int, Int, Int) = read()
domain Sentence = 1..=n
let words: [Sentence]Str = read()

var ends: [0..=n]Int = 0
for s in Sentence: ends[s.label] = ends[s.label - 1] + words[s].len + 1

var best: [0..=n]F64 = 0.0
var cut: [0..=n]Int = 0
fn line(j: Int, i: Int) -> Int = abs(ends[i] - ends[j] - 1 - l)
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** F64(p)

var wins: List<(Int, Int)> = [(1, 0)]
var now = 0
for i in 1..=n:
    while now + 1 < wins.len and wins[now + 1].0 <= i: now += 1
    cut[i] = wins[now].1
    best[i] = through(cut[i], i)
    while wins.last!.0 > i and through(i, wins.last!.0) <= through(wins.last!.1, wins.last!.0):
        wins.pop()
    let prev = wins.last!.1
    let start = max(wins.last!.0, i) + 1
    if let pos = bsearch_first(start..=n, mid => through(i, mid) <= through(prev, mid)):
        wins.push((pos, i))

if best[n] > 1e18:
    print("Too hard to arrange")
else:
    var breaks = [n]
    while breaks.last! > 0: breaks.push(cut[breaks.last!])
    breaks.reverse()
    print(sum(line(breaks[k], breaks[k + 1]) ** p for k in 0..breaks.len - 1))
    for k in 0..breaks.len - 1:
        print([words[Sentence(i)] for i in breaks[k] + 1..=breaks[k + 1]])
print(dashes)
