// Mutations in a traced run: the VM makes each one and reports its step.
var seen: Set<Int> = Set()
var log: List<Int> = []
fn collect(n: Int) -> List<Int>:
    var xs: List<Int> = []
    for i in 0..n:
        xs.push(i * i)
        seen.insert(i % 3)
    xs.reverse()
    xs
let n: Int = read()
var order = collect(n)
order.push(7)
log.push(order.len)
print(order, seen.len, log)
// A generator's yields still drive the loop that runs it.
var dq = Deque<Int>()
dq.push(5)
dq.push_front(4)
for (i, x) in dq.items: print(i, x)
