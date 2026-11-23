// expect: ok, output "YES"
var q = Deque.of([2])
q.push_front(1)
print(yes(q.to_list() == [1, 2]))
