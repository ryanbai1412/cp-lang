var q: Deque<Int> = []
q.push(1)
q.push(2)
q.push_front(9)
let (n, mid, f, l) = (q.len, q[1], q.first!, q.last!)
let a = q.pop_front()!
let b = q.pop()!
let c = q.pop()!
print(n, mid, f, l, c, a, b, q.pop(), q.len, q.is_empty, none: "none")
