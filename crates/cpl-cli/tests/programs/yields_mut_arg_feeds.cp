// `for x in f(mut v)` over a `yields` fn that changes its argument, or
// calls a fn value, runs its body as `f` yields in both engines: leaving
// the loop early leaves the rest of the work undone (R13.3.3).
fn take_all(mut xs: List<Int>, keep: Int -> Bool) -> yields Int:
    while xs.len > 0:
        let x = xs.pop()!
        if keep(x): yield x
var h = Heap.of([5, 3, 1, 4, 2])
for x in h.drain():
    if x == 2: break
print(h.len)
var st = [1, 2, 3, 4, 5, 6]
for x in take_all(mut st, x => x % 2 == 0):
    print("got", x)
    if x == 4: break
print(st)
