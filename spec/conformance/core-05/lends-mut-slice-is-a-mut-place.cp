// expect: ok, output "5 6 7 8 9\n6 5 9 8 7\n9 5 6 7 8"
// rule: 05.1.1
// rule: 05.6.3
// A `lends mut` slice is a place like any other: a generic fn that changes its `mut` argument, or a method that changes its receiver, writes through it into the source.
domain V = 0..5
struct Bag:
    cnt: [V]Int
    xs: List<Int>

    fn window(self, l: V, r: V) -> lends mut [V]Int:
        @law disjoint(l, r, l2, r2) = r < l2 or r2 < l
        mut cnt[l..=r]

    fn part(self, l: Int, r: Int) -> lends mut List<Int>:
        @law disjoint(l, r, l2, r2) = r < l2 or r2 < l
        mut xs[l..r]

var b = Bag([V] v => 9 - Int(v), [9, 8, 7, 6, 5])
b.window(V(0), V(4)).sort()
print(b.cnt)
reverse(mut b.part(1, 5))
b.part(1, 5).sort()
reverse(mut b.window(V(1), V(4)))
reverse(mut b.window(V(0), V(4)))
reverse(mut b.window(V(1), V(4)))
print(b.cnt)
print(b.xs)
