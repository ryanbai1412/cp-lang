use std.sparse.{SparseTable}
use std.segtree.{SegTree}

struct Box<D>:
    a: [D]Int

    pub fn first_at_least(self, x: Int) -> ?D:
        for i in D:
            if a[i] >= x: return Some(i)
        None

    pub fn at(self, i: D) -> Int = a[i]

    pub fn part(self, r: Range<D>) -> Int = sum(a[i] for i in r)

    pub fn all(self) -> [D]Int = a

fn total<D>(a: [D]Int) -> Int = sum(a[i] for i in D)

fn rev_of<D>(a: [D]Int) -> List<D> = [i for i in D.rev]

let n: Int = read()
let b = Box([1..=n] i => i * i)
print(b.first_at_least(10) ?? -1, b.at(2), b.part(2..4), b.part(2..=n), b.all())
let w = b.all()
print(w[1], w[n])
let c = Box([0..n] i => i + 1)
let j = c.first_at_least(3) ?? -1
print(j + 1, c.at(j), c.part(0..n))
print(rev_of(w))
let seg = SegTree(max, [0..n] i => (i * 3) % 5)
print(seg.fold(0..n), seg.fold(1..3))
var calls = 0
fn bump() -> Int:
    calls += 1
    calls
print(total([0..n] i => i + bump()), calls)
print(b.at(n + 1))
