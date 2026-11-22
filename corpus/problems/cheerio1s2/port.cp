// Cheerio Contest 1 S2, n cheapest seats (buildable today-form)
// original: corpus/submissions/5337643.cpp
let n: 1..=1e5 = read()
let a: 1..=1e4 = read()
let b: 1..=1e4 = read()

var next = MinHeap<Int>()
for i in 1..=n: next.push(a * i + b)
var total = 0
for _ in 0..n:
    let w = next.pop()!
    total += w
    next.push(w + b)
print(total)
