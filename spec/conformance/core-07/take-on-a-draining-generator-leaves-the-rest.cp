// expect: ok, output "1\n2\n4 5\n1\n0\n3\nYES 1\n5 0"
// rule: 07.7.3
// `take(k)` on a generator method pulls exactly `k` values, so a draining one leaves the
// values it never yields in its source; a `for` that breaks, `any` and `first` stop pulling
// the same way (R07.1.3).
var h = Heap.of([3, 1, 2])
for x in h.drain().take(1): print(x)
print(h.len)
var g = Heap.of([5, 4, 6])
print(g.drain().take(2).collect())
print(g.len)
print(g.drain().take(0).collect().len)
var q = Heap.of([3, 1, 2, 5, 4])
for x in q.drain():
    if x == 2: break
print(q.len)
print(yes(q.drain().any(x => x == 4)), q.len)
print(q.drain().first!, q.len)
