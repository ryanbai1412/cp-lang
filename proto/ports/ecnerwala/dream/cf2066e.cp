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

var count: [Vol]Int = 0
var total = 0
var unique = SortedSet<Vol>()                  // amounts held by exactly one barrel
var alone = Fenwick<Vol>()                     // at each unique amount: that amount
var gap = SegTree<Vol, ?Int>(min)              // at each unique amount: gap to the next one

fn put(v: Vol, delta: -1..=1):
    count[v] += delta
    total += delta * v
    let is_alone = count[v] == 1
    if is_alone == (v in unique): return
    unique.toggle(v)
    alone[v] = if is_alone: v else: 0
    gap[v] = if is_alone: unique.after(v) - v      // none at the heaviest
    if let u = unique.before(v): gap[u] = unique.after(u) - u

fn identifiable() -> Bool:
    let shared = total - alone.sum()
    var next = unique.first
    while let b = next and b != unique.last:
        let have = shared + alone.sum(..b)         // once everything lighter is checked
        if b > have and gap.min(b..)! > have: return false
        next = unique.first(above: max(have, b))
    true

let (n, q): (1..=2e5, 1..=2e5) = read()
for v in read<[n]Vol>(): put(v, +1)
print(identifiable(), as: .yes_no)
repeat q:
    let (op, v): ('+' | '-', Vol) = read()
    put(v, if op == '+': +1 else: -1)
    print(identifiable(), as: .yes_no)
