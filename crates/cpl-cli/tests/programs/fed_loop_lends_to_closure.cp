// A loop fed by a `yields` fn that takes a `mut` place and a closure that
// reads it: the fed fn lends the place on, so each closure call sees its
// latest value, a global's or a local's.
struct Bag:
    xs: List<Int>

fn grow(mut self: Bag, n: Int, f: Int -> Int) -> yields Int:
    for i in 0..n:
        self.xs.push(f(i))
        yield self.xs.len

var g = Bag(xs: [1])
var total = 0
for k in g.grow(3, x => x * 10 + g.xs.len):
    total += k
print(g.xs, total)

fn local_lend() -> Int:
    var b = Bag(xs: [5])
    var s = 0
    for k in b.grow(2, x => x + b.xs[b.xs.len - 1]):
        s += k
    s * 1000 + b.xs[1] * 10 + b.xs[2]

print(local_lend())
