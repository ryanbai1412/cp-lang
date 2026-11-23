var h = MinHeap<Int>()
var xs = [5]
for x in [3, 8, 1]:
    xs.push(x)
    let z = xs.pop()!
    h.push(x)
    print(z)
    let y = h.pop()!
    print(y)
