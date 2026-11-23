let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var h = MaxHeap<Int>()
for x in a: h.push(x)
for x in h.drain().take(k):
    print(x, end: " ")
