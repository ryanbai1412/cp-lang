// `Heap.new`, `push` and `pop` are `todo` stubs: `cpl run` traps, and the C++ build does not compile.
var h = Heap<Int>.new()
h.push(3)
h.push(1)
print(h.pop()!)
