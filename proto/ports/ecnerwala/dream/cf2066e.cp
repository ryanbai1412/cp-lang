// Codeforces 2066 E: Tropical Season (dream).
// Two equal barrels either expose the poison or are both safe, so every barrel
// whose amount repeats is safe water. Check the barrels with a unique amount,
// lightest first. With `have` kilograms of safe water, a barrel b can be checked
// if b <= have (pour until equal, compare), and two unchecked barrels can be if
// their gap is <= have (top up the lighter, compare); each check adds to `have`.
// Identification works iff this never gets stuck while two or more barrels are
// unchecked. It can only get stuck at a b with b > have, and `have` more than
// doubles between two such points, so there are O(log) of them; at each, some
// gap among the heavier unique barrels must fit.
type Vol = 1..=1e6

// Maintains the multiset of barrel volumes, the unique volumes with their
// prefix sums, and the minimum gap between consecutive unique volumes.
struct Barrels:
    count: [Vol]Int = 0
    total: Int = 0
    unique: SortedSet<Vol> = SortedSet()
    gaps: SegTree<Vol, ?Int>(min) = SegTree(min)

    fn put(mut self, v: Vol, delta: -1..=1):
        count[v] += delta
        total += delta * v
        let is_alone = count[v] == 1
        if is_alone == (v in unique): return
        unique.toggle(v)
        gaps[v] = if is_alone: unique.after(v) - v          // none at the heaviest
        if let u = unique.before(v): gaps[u] = unique.after(u) - u

    fn safe_before(self, b: Vol) -> Int = (total - unique.sum(..)) + unique.sum(..b)

    fn identifiable(self) -> Bool:
        let stuck_points = iterate(
            from: unique.first,
            step: b => unique.first(above: max(self.safe_before(b), b)),
        )
        for b in stuck_points where b != unique.last:
            let have = self.safe_before(b)
            if b > have and gaps.min(b..)! > have: return false
        true

var barrels = Barrels()
let (n, q): (1..=2e5, 1..=2e5) = read()
for v in read<[n]Vol>(): barrels.put(v, +1)
print(barrels.identifiable(), as: .yes_no)
repeat q:
    let (op, v): ('+' | '-', Vol) = read()
    barrels.put(v, if op == '+': +1 else: -1)
    print(barrels.identifiable(), as: .yes_no)
