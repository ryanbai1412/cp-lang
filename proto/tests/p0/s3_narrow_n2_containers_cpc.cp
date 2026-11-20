// Synthesis #3 Leap 2 (Rule N2 on Heap and Deque, compiled cpc path).
var pq: MaxHeap<Int> = []
pq.push(5)
pq.push(2)
pq.push(9)
var hsum = 0
while not pq.is_empty:
    let top: Int = pq.peek
    let popped: Int = pq.pop()
    hsum += top + popped

var dq: Deque<Int> = []
dq.push(10)
dq.push(20)
dq.push(30)
var dsum = 0
while dq.len > 0:
    let f: Int = dq.first
    let v: Int = dq.pop_front()
    dsum += f + v

print(hsum, dsum)
