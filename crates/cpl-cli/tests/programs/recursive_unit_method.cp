// A method with no `->` that ends by calling itself returns nothing, and
// can be called before it is declared.
struct Tree:
    v: List<Int>

    fn make(n: Int) -> Tree:
        var t = Tree(List.filled(4 * n, 0))
        t.build(1, 0, n - 1)
        t

    fn build(mut self, x: Int, l: Int, r: Int):
        v[x] = r - l + 1
        if l == r: return
        let m = (l + r) / 2
        self.build(2 * x, l, m)
        self.build(2 * x + 1, m + 1, r)

fn countdown(n: Int):
    if n == 0: return
    print(n)
    countdown(n - 1)

let t = Tree.make(4)
print(t.v)
countdown(3)
