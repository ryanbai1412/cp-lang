// Bounds: n: 1..=2e5; people.need: 0..=n; people.price: 0..=1e4

let n: 1..=2e5 = read()
let people: [n](need: 0..=n, price: 0..=1e4) = read()

var by_need: [0..=n]List<0..=1e4> = []
for p in people: by_need[p.need].push(p.price)

var waiting = MinHeap<Int>()
var paid = 0
for k in (0..=n).rev:
    for price in by_need[k]: waiting.push(price)
    while n - waiting.len < k: paid += waiting.pop()!
print(paid)
