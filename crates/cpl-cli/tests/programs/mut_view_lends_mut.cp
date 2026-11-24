// A `lends mut` method's slice is a place: verbs and calls taking it
// `mut` write into the struct's own list, from a fn and from top level.
struct Bag:
    xs: List<Int>

    fn part(self, l: Int, r: Int) -> lends mut List<Int>:
        @law disjoint(l, r, l2, r2) = r < l2 or r2 < l
        mut xs[l..r]

fn shuffle(mut b: Bag):
    b.part(0, 3).sort()
    reverse(mut b.part(3, 6))
var b = Bag([6, 5, 4, 3, 2, 1])
shuffle(mut b)
print(b.xs)
reverse(mut b.part(1, 5))
print(b.xs)
