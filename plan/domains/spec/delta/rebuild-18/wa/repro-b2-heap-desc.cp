var h: Heap<Int> = Heap(by: x => desc(x))
h.push(1)
h.push(3)
print(h.pop()!)
