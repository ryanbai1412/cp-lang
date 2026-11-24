// A `yields` fn whose own loop is fed by another `yields` fn: each level
// feeds the next, and a `break` or `return` in the outermost body stops all.
fn upto(n: Int) -> yields Int:
    var i = 0
    while i < n:
        yield i
        i += 1

fn squares(n: Int) -> yields Int:
    for x in upto(n): yield x * x

fn pairs(n: Int) -> yields (a: Int, b: Int):
    for x in squares(n):
        for y in upto(2): yield (a: x, b: y)

fn bump_all(mut total: Int, n: Int) -> yields Int:
    for x in squares(n):
        total += x
        yield total

fn first_over(n: Int, limit: Int) -> Int:
    for x in squares(n):
        if x > limit: return x
    -1

var seen: List<Int> = []
for x in squares(5): seen.push(x)
print(seen)
for x in squares(10):
    if x > 20: break
    if x % 2 == 1: continue
    print(x)
for p in pairs(3): print(p.a, p.b)
var t = 100
for s in bump_all(mut t, 4): print(s)
print(t)
print(first_over(10, 30), first_over(3, 30))
var ss = SortedSet<Int>.of([5, 1, 3])
print([x for x in ss])
var acc = 0
for x in ss:
    acc = acc * 10 + x
    if x == 3: break
print(acc)
