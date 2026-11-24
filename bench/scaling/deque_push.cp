// scaling: deque.Deque.push
let n: Int = read()
var d = Deque<Int>()
for i in 0..8 * n:
    d.push(i) // timed
print(d.len)
