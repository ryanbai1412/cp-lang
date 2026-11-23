// Source: Codeforces 2066E (Tropical Season), https://codeforces.com/contest/2066/problem/E
// Problem: n barrels, a_i <= 1e6, one secretly poisoned. Scales compare any two barrels, and water
//   may be poured from a barrel known to be safe. Can the poisoned barrel always be identified?
//   Answer Yes/No before and after each of q queries "+ x" / "- x" (add/remove a barrel);
//   n, q <= 2e5.
// Solution: barrels whose amount repeats are safe. Check the unique amounts lightest first: with
//   `have` kilograms of safe water, b is checkable if b <= have, or two unchecked barrels if their
//   gap is <= have. It can only get stuck at a unique b > have, and then `have` at least doubles,
//   so O(log sum) stuck points; at each, test the smallest gap among unique amounts >= b. Counts,
//   a SortedSet of unique amounts, a Fenwick of their sums and a min SegTree of gaps give
//   O(log V log sum) per answer.

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
var gap = SegTree<Vol, ?Int>(min, fill: None)

fn refresh_gap(u: Vol):
    gap.set(u, if let nxt = u.next: unique.range(nxt..).first.map(w => w - u) else: None)

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
        gap.set(v, None)
    if let u = unique.range(..v).last: refresh_gap(u)

fn safe_before(b: Vol) -> Int = total - unique_sum.fold(b..)

fn identifiable() -> Bool:
    let last = unique.last ?? return true
    var at = unique.first
    while let b = at and b != last:
        let have = safe_before(b)
        if b.ord > have and gap.fold(b..)! > have: return false
        let next_v = max(have, b.ord) + 1
        at = if next_v <= max_v: unique.range(Vol(next_v)..).first else: None
    true

let (n, q): (1..=200_000, 1..=200_000) = read()
for v in read<[n]1..=max_v>(): put(Vol(v), Delta.Add)
print(if identifiable(): "Yes" else: "No")
repeat q:
    let (op, v): (Char, 1..=max_v) = read()
    put(Vol(v), if op == '+': Delta.Add else: Delta.Remove)
    print(if identifiable(): "Yes" else: "No")
