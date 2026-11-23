// CCO 2019 P6 Bad Codes: n different codewords of 0s and 1s (n <= 50, each at
// most m <= 50 long). A message is codewords written one after another.
// Print the length of the shortest message that can be read as two different
// lists of codewords, or -1 if there is none.
//
// Write the two readings side by side, one codeword at a time on whichever
// side is behind. What matters is the part by which one side is ahead: a
// tail of some codeword, so at most n * m states. Putting a codeword on the
// short side costs the letters that become common to both. Dijkstra from
// every pair of codewords where one starts the other, to the empty tail.
// original: corpus/submissions/3617414.cpp

fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head

/// Put `b` beside `a`, both starting at the same place: what sticks out, and how much is shared.
fn overhang(a: Str, b: Str) -> ?(Str, Int):
    if starts_with(a, b): Some((a[b.len..a.len], b.len))
    elif starts_with(b, a): Some((b[a.len..b.len], a.len))
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
