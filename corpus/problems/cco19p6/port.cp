// Source: CCO 2019 Day 2 P3 (CCO '19 P6), Bad Codes, https://dmoj.ca/problem/cco19p6
// Problem: n distinct binary codewords (n <= 50), each 1..m bits long (m <= 50). Print the
//   length of the shortest bit string that splits into codewords in two different ways,
//   or -1 if every bit string decodes uniquely.
// Solution: Write the two readings side by side, adding a codeword to whichever is behind;
//   only the part by which one side is ahead matters, a tail of some codeword, so there are
//   at most n * m states. A codeword placed against the tail costs the bits now shared by
//   both. Dijkstra (map of distances, min-heap) starts from every pair of codewords where
//   one is a prefix of the other and stops at the empty tail: O(n^2 m^2 log(nm)).
// original: corpus/submissions/3617414.cpp

fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head

/// Put `b` beside `a`, both starting at the same place: what sticks out, and how much is shared.
fn overhang(a: Str, b: Str) -> ?(Str, Int):
    if starts_with(a, b): Some((a[b.len..a.len].to_str(), b.len))
    elif starts_with(b, a): Some((b[a.len..b.len].to_str(), a.len))
    else: None

let (n, m): (Int, Int) = read()
domain Word = 1..=n
let words: [Word]Str = read()

var dist = Map<Str, Int>()
var queue = MinHeap<(Int, Str)>()
fn reach(tail: Str, d: Int):
    if d < (dist.get(tail) ?? Int.MAX):
        dist[tail] = d
        queue.push((d, tail))

for i in Word:
    for j in Word:
        if i < j:
            if let (tail, shared) = overhang(words[i], words[j]): reach(tail, shared)

var answer = -1
while not queue.is_empty:
    let (d, tail) = queue.pop()!
    if d != dist[tail]: continue
    if tail.is_empty:
        answer = d
        break
    for w in Word:
        if let (next, shared) = overhang(tail, words[w]): reach(next, d + shared)
print(answer)
