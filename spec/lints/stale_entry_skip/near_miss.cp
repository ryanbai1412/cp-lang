let n: 1..=100 = read()
let limit: 0..=1000 = read()
let a: [0..n]Int = read()
var pq = MinHeap<(Int, Int)>()
for i in 0..n: pq.push((a[i], i))
var taken = 0
while let (d, i) = pq.pop():
    if d > limit:
        break
    taken += 1
print(taken)
