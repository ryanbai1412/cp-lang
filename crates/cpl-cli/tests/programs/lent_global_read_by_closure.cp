// A closure passed beside a `mut self` receiver reads the lent place while
// the method runs: each closure call sees the receiver's latest value.
struct Bag:
    xs: List<Int>

fn bump(mut self: Bag, f: Int -> Int):
    self.xs.push(f(self.xs.len))
    self.xs[0] = f(self.xs[0])

fn grow(mut self: Bag, f: Int -> Int) -> Int:
    self.xs.push(0)
    f(self.xs.len) + self.xs.len

var g = Bag(xs: [1])
g.bump(x => x + g.xs.len * 100)
print(g.xs)
print(g.grow(x => x * 10 + g.xs[g.xs.len - 1]))
print(g.xs)
var hs: HashMap<Int, Int> = HashMap<Int, Int>()
for i in 0..4:
    hs.update(i % 2, -1, v => if v < 0: hs.len * 10 else: v + hs.len)
print(hs[0], hs[1], hs.len)

fn local_lend() -> Int:
    var b = Bag(xs: [5, 6])
    b.bump(x => x + b.xs.len)
    b.xs[0] + b.xs[1] * 10 + b.xs[2] * 100

print(local_lend())
