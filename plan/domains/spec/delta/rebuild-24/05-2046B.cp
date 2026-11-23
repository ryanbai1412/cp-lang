// CF 2046B Move Back at a Cost. An element stays only if nothing after it is smaller:
// the stack of suffix minima. Everything popped moves and gains 1; a kept value above
// the smallest moved value must move too. The result is the sorted multiset.
@!multitest
let n: 1..=1e5 = read()
let a = read<[n]1..=1e9>()

var kept: List<Int> = []
var out: List<Int> = []
for x in a:
    while not kept.is_empty and kept.last! > x: out.push(kept.pop()! + 1)
    kept.push(x)
var low: ?Int = None
for x in out: low min= x
for x in kept:
    out.push(if low.is_some and x > low!: x + 1 else: x)
out.sort()
print(out)
