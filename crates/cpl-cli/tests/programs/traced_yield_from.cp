// `yield from` in a `yields` fn feeds the loop each item in turn, traced
// or not: a `break` or `continue` in the body acts on that item alone.
fn both(a: List<Int>, b: [3]Int) -> yields Int:
    yield from a
    yield 0
    yield from b

fn twice(a: List<Int>) -> yields Int:
    yield from a
    for x in a:
        if x > 5: return
    yield from a

for x in both([1, 2], [3, 4, 5]):
    if x == 2: continue
    if x == 4: break
    print("both", x)
var got: List<Int> = []
for x in twice([5, 50, 80, 9]):
    got.push(x)
    if x == 80: break
print(got)
print([x for x in both([6], [1, 2, 3])])
