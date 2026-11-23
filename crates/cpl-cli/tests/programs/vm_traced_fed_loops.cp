// A traced run feeds a loop from a `yields` fn on the VM: a fn value that
// cannot see the lent argument, a `return` that pops, a `let` that pops,
// and lent globals and cells, all as the untraced run and the C++ do.
fn take_all(mut xs: List<Int>, keep: Int -> Bool) -> yields Int:
    while xs.len > 0:
        let x = xs.pop()!
        if keep(x): yield x
fn pop_all(mut xs: List<Int>) -> yields Int:
    while xs.len > 0: yield xs.pop()!
fn last_of(mut xs: List<Int>) -> ?Int:
    return xs.pop()
var st = [1, 2, 3, 4, 5, 6]
for x in take_all(mut st, x => x % 2 == 0):
    print("got", x)
    if x == 4: break
print(st)
var limit = 3
var g = [7, 8, 9, 10]
for x in take_all(mut g, x => x > limit + 5):
    print("kept", x)
print(g)
var rows = [[1, 2], [3, 4, 5]]
var total = 0
for x in pop_all(mut rows[1]):
    total += x
print(total, rows)
for x in pop_all(mut g): print("rest", x)
print(last_of(mut st) ?? 0, st)
