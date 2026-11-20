// Cheerio Contest 1 S2, n cheapest seats
// original: corpus/submissions/5337643.cpp
// Seat j of row i costs a * i + b * j. Each row is an increasing stream, so
// the heap holds every row's next price; the row itself is never needed,
// because the seat after one that costs w costs w + b.
let (n, a, b): (1..=1e5, 1..=1e4, 1..=1e4) = read()

var next = MinHeap<Int>()
for i in 1..=n: next.push(a * i + b)
var total = 0
for _ in 0..n:
    let w = next.pop()!
    total += w
    next.push(w + b)
print(total)
