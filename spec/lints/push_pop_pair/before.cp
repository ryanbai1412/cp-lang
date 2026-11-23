var h = MinHeap<Int>()
h.push(5)
for x in [3, 8, 1]:
    h.push(x)
    let y = h.pop()!
    print(y)
