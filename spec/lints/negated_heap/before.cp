let a = [3, 1, 4]
var h = MinHeap<Int>()
for x in a:
    h.push(-x)
print(-(h.pop()!))
