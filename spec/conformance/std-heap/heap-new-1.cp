// expect: ok, output "YES"
var h: Heap<Int> = Heap()
h.push(3)
h.push(1)
print(yes(h.pop() == Some(1)))
