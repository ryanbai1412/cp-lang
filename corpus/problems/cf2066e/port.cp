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

enum Delta: Add | Remove

let max_v = 1_000_000
domain Vol = 0..=max_v

// Before checking unique barrel b, the safe water is everything except the
// unique barrels >= b. gap[u] is the distance from unique u to the next one
// (none when u is not unique or is the largest).
var count: [Vol]Int = 0
var total = 0
var unique = SortedSet<Vol>()
var unique_sum = Fenwick<Vol, Int>()
var gap = SegTree<Vol, ?Int>(min, fill: none)

fn refresh_gap(u: Vol):
    gap.set(u, if let nxt = u.next: unique.range(nxt..).first.map(w => w - u) else: none)

fn put(v: Vol, d: Delta):
    let sign = match d:
        Delta.Add => 1
        Delta.Remove => -1
    let was_unique = count[v] == 1
    count[v] += sign
    total += sign * v.ord
    let is_unique = count[v] == 1
    if is_unique == was_unique: return
    if is_unique:
        unique.add(v)
        unique_sum.add(v, v.ord)
        refresh_gap(v)
    else:
        unique.remove(v)
        unique_sum.add(v, -v.ord)
        gap.set(v, none)
    if let u = unique.range(..v).last: refresh_gap(u)

fn safe_before(b: Vol) -> Int = total - unique_sum.fold(b..)

fn identifiable() -> Bool:
    let last = unique.last ?? return true
    var at = unique.first
    while let b = at and b != last:
        let have = safe_before(b)
        if b.ord > have and gap.fold(b..)! > have: return false
        let next_v = max(have, b.ord) + 1
        at = if next_v <= max_v: unique.range(Vol(next_v)..).first else: none
    true

let (n, q): (1..=200_000, 1..=200_000) = read()
for v in read<[n]1..=max_v>(): put(Vol(v), Delta.Add)
print(if identifiable(): "Yes" else: "No")
repeat q:
    let (op, v): (Char, 1..=max_v) = read()
    put(Vol(v), if op == '+': Delta.Add else: Delta.Remove)
    print(if identifiable(): "Yes" else: "No")
