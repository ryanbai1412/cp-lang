var h = MinHeap<Int>()
h.push(5)
for x in [3, 8, 1]:
    let y = h.push_pop(x)
    print(y)
