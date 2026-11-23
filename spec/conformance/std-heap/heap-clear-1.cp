// expect: ok, output "YES"
var h = Heap.of([1, 2])
h.clear()
print(yes(h.pop() == None))
