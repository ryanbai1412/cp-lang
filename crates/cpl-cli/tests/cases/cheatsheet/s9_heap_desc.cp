var h = MinHeap<(Int, Desc<Int>)>()
h.push((1, desc(2)))
h.push((1, desc(9)))
h.push((0, desc(0)))
let _ = h.pop()
let (a, b) = h.pop()!
var mx = MaxHeap<Int>()
mx.push(2); mx.push(3)
print(a, b.val, h.len, mx.len, mx.peek!)
