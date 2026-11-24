// expect: ok, output "1 3 5 7 9 11 0 2 4 6 8 10\n2 6 4 2 6 4 6 3 9 6 3 9\n4 6 2 4 6 2 9 3 6 9 3 6\n6 3 9 6 3 9 2 6 4 2 6 4\n2 6"
// rule: 07.11.1
// Every sort is stable under a user `Ord` whose ties are distinguishable: by a key, descending, reversed after, in place on a `[D]T`, and `max`/`min` give the first of equal elements.
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(k % 3 + 1, (k % 3 + 1) * (k % 2 + 2)) for k in 0..12]
let tagged = [(xs[k], k) for k in 0..12]
print([t.1 for t in tagged.sorted(by: t => t.0)])
print([f.q for f in xs.sorted(by: x => desc(x))])
print([f.q for f in xs.sorted().rev])
domain D = 0..12
var a: [D]Frac = Frac(1, 1)
for d in D:
    a[d] = xs[d.ord]
a.sort()
print([f.q for f in a])
print(xs.max()!.q, xs.min()!.q)
