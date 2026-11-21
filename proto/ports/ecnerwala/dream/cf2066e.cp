// Codeforces 2066 E: Tropical Season.
// Two equal barrels either expose the poison or are both safe, so every barrel
// whose amount repeats is safe water. Check the barrels with a unique amount,
// lightest first. With `have` kilograms of safe water, a barrel b can be checked
// if b <= have (pour until equal, compare), and two unchecked barrels can be if
// their gap is <= have (top up the lighter, compare); each check adds to `have`.
// Identification works iff this never gets stuck while two or more barrels are
// unchecked. It can only get stuck at a unique barrel b with b > have; at the
// next unique barrel b' > b, `have` has grown by at least b > have, so `have`
// more than doubles every two steps and there are O(log(total)) stuck points.
// At each, some gap among the unique barrels >= b must be <= have.

domain Vol = 1..=1_000_000

// Maintains the multiset of barrel volumes alongside the set of unique volumes,
// their prefix sums, and range minimum gaps between consecutive unique volumes.
struct Barrels:
    all: SortedBag<Vol> = SortedBag()
    unique: SortedSet<Vol> = SortedSet()

    fn put(mut self, v: Vol, delta: -1..=1):
        if delta == 1: all.add(v) else: all.remove(v)
        if all.count(v) == 1: unique.add(v) else: unique.remove(v)

    fn safe_before(self, b: Vol) -> Int = (all.sum() - unique.sum(..)) + unique.sum(..b)

    fn identifiable(self) -> Bool:
        let stuck_points = iterate(
            from: unique.first,
            step: b => unique.range(max(self.safe_before(b), b) + 1..).first,
        )
        for b in stuck_points if b != unique.last:
            let have = self.safe_before(b)
            if b > have and unique.min_gap(b..)! > have: return false
        true

var barrels = Barrels()
let (n, q): (1..=200_000, 1..=200_000) = read()
for v in read<[n]Vol>(): barrels.put(v, +1)
print(barrels.identifiable(), as:.yes_no)
repeat q:
    let (op, v): ('+' | '-', Vol) = read()
    barrels.put(v, if op == '+': +1 else: -1)
    print(barrels.identifiable(), as:.yes_no)
