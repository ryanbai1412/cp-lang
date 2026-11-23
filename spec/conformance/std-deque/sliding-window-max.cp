// expect: ok, output "3 3 5 5 6 7"
let a = [1, 3, -1, -3, 5, 3, 6, 7]
let k = 3
var q: Deque<Int> = Deque()
var out: List<Int> = []
for i in 0..a.len:
    while not q.is_empty and a[q.last!] <= a[i]: q.pop()
    q.push(i)
    if q.first! <= i - k: q.pop_front()
    if i >= k - 1: out.push(a[q.first!])
print(out)
