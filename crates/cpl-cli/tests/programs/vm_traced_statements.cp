// A traced run steps through each statement on the VM, as the tree-walker
// did: asserts, global writes, calls, unwraps, `while`, prints, a write
// inside a `let`, and a loop over a range that never runs.
var total = 0
var xs: List<Int> = [4, 7, 1]
fn add(k: Int) -> Int:
    total = total + k
    return total
fn half(o: ?Int) -> ?Int:
    let v = o? / 2
    return v
let a = add(3)
assert a == 3, "add"
let last = xs.pop()!
let h = half(Some(last))!
var i = 0
while i < xs.len:
    total = total + xs[i]
    i += 1
domain D = 1..=5
let r: Range<D> = D(1)..=D(5)
print([x for x in r.take(2)])
print([x for x in r.drop(10)])
print(a, last, h, total, half(None) ?? -1)
