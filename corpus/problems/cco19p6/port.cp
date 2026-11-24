// Source: CCO 2019 Day 2 P3 (CCO '19 P6), Bad Codes, https://dmoj.ca/problem/cco19p6
// limits: 600 ms, 1024 MB
// Problem: n distinct binary codewords (n <= 50), each 1..m bits long (m <= 50). Print the
//   length of the shortest bit string that splits into codewords in two different ways,
//   or -1 if every bit string decodes uniquely.
// Solution: write the two readings side by side, adding a codeword to whichever is behind.
//   Only the part by which one side is ahead matters, and it is a tail of some codeword: a
//   `Tail` (word, cut), at most n * (m + 1) of them. A codeword laid against the tail costs
//   the bits now shared by both. Std `dijkstra` from every pair of codewords where one is a
//   prefix of the other; the answer is the nearest empty tail. O(n^2 m^2 log(nm)).
// original: corpus/submissions/3617414.cpp
// Bounds: n: 1..=50; m: 1..=50
let (n, m): (1..=50, 1..=50) = read()
domain Word = 1..=n
domain Cut = 0..=m
domain Tail = (Word, Cut)  // words[w] from bit `cut` on
let words: [Word]Str = read()

fn rest(t: Tail) -> Str = words[t.0][Int(t.1)..words[t.0].len].to_str()
fn ended(t: Tail) -> Bool = Int(t.1) == words[t.0].len

// Lay word `x` against the tail `t`: the new tail and the bits both readings now share.
fn lay(t: Tail, x: Word) -> ?(to: Tail, w: Int):
    let (s, word) = (rest(t), words[x])
    if word.starts_with(s): Some((to: (x, Cut(s.len)), w: s.len))
    elif s.starts_with(word): Some((to: (t.0, Cut(Int(t.1) + word.len)), w: word.len))
    else: None

var arcs: List<(from: Tail, arc: (to: Tail, w: Int))> = []
for t in Tail if Int(t.1) < words[t.0].len:
    for x in Word:
        if let a = lay(t, x): arcs.push((from: t, arc: a))
var starts: List<(Tail, Int)> = []
for (i, j) in Word.pairs():
    if let a = lay((i, Cut(0)), j): starts.push((a.to, a.w))
let dist = dijkstra(Graph.of(arcs), from: starts)
print(Tail.filter_map(t => if ended(t): dist[t] else: None).min() ?? -1)
