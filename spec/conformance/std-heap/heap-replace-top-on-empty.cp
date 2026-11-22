// expect: ok, output "YES 7"
// rule: std.heap.Heap.replace_top
var h = Heap<Int>()
print(yes(h.replace_top(7) == None), h.peek!)
