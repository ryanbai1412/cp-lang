// Source: NOI '09 P2 (Little G the Poet), https://dmoj.ca/problem/noi09p2
// Problem: N <= 1e5 sentences (length <= 30), in order, set into lines joined by single
//   spaces; a line of length x costs |x - L|^P (L <= 3e6, P <= 10). Minimise the total: print
//   it and the lines, or "Too hard to arrange" if it exceeds 1e18; then 20 dashes. Multitest.
// Solution: best[i] = min over j of best[j] + |len(j+1..i) - L|^P; the cost is convex, so
//   decisions are monotone. `wins` is a queue of (first i, decision j): a new decision pops
//   entries it beats at their start, then binary-searches (`first_true`) where it overtakes
//   the last one. Compared in F64, total printed exactly from the `cut` chain: O(N log N).
// original: corpus/submissions/5108899.cpp

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
    if let pos = (start..=n).first_true(mid => through(i, mid) <= through(prev, mid)):
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
