// `for x in f(..)` over a `yields` fn runs its body as `f` yields, in
// `cpl run` as in the C++: a fn that never stops yielding can be left
// with `break` or `return`, and what the body printed is out before the
// fn goes on.
fn nat() -> yields Int:
    var i = 0
    while true:
        yield i
        i += 1

fn evens(k: Int) -> yields Int:
    for x in 0..k:
        if x % 2 == 0: yield x

fn both(a: List<Int>, b: List<Int>) -> yields Int:
    yield from a
    yield from b

fn first_square_over(bar: Int) -> Int:
    for x in nat():
        if x * x > bar: return x
    -1

for x in nat():
    if x > 3: break
    print(x)
print(first_square_over(50))
for x in evens(7): print("even", x)
var seen = 0
for a in nat():
    if a > 5: break
    for b in nat():
        if b > a: break
        if b == 1: continue
        seen += 1
print(seen)
for x in both([1, 2], [3]):
    if x == 2: continue
    print("both", x)
let n: Int = read()
var cells: [0..4]Int = 0
for x in nat():
    print("at", x)
    cells[x * n] += 1
