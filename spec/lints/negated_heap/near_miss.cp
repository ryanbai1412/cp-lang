let a = [3, 1, 4]
var xs: List<Int> = []
for x in a:
    xs.push(-x)
var h = MinHeap<Int>()
for x in a:
    h.push(x - 1)
print(xs, h.pop()!)
