domain D = read(1..=500_000)
var pq = MinHeap<(D, D)>()
pq.push((D.last!, D.last!))
let (x, y) = pq.pop()!
print(yes(x == D.last!), yes(y == D.last!))
