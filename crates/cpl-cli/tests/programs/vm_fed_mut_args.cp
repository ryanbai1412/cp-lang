// A `for` over a `yields` fn that takes `mut` arguments runs on the VM:
// each argument is lent to the fn for the whole loop and is back in its
// place after the loop, however the loop ends.
fn pop_all(mut xs: List<Int>) -> yields Int:
    while xs.len > 0: yield xs.pop()!
fn count_up(mut k: Int, to: Int) -> yields Int:
    while k < to:
        k += 1
        yield k
fn first_big(mut xs: List<Int>, at_least: Int) -> Int:
    for x in pop_all(mut xs):
        if x >= at_least: return x
    -1
struct Bag:
    var items: List<Int>
let n: Int = read()
var h = Heap.of([n, 3, 9, 1])
var sum = 0
var k = 0
for v in count_up(mut k, n):
    if v % 2 == 0: continue
    sum += v
print(sum, k)
var bag = Bag(items: [1, 2, 3, 4])
for x in pop_all(mut bag.items):
    if x == 2: break
print(bag.items)
var local = [10, 20, 30]
let big = first_big(mut local, 15)
print(big, local)
for x in h.drain():
    if x > 5: break
    print(x)
print(h.len)
