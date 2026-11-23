// expect: ok, output "YES"
var q = Deque.of([1, 2, 3])
q.rotate_right(4)
print(yes(q.to_list() == [3, 1, 2]))
