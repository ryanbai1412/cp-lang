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
use std.bag.{AugSortedBag, AugSortedSet}

domain Vol = 1..=1_000_000

// Unique amounts lo..hi in ascending order: their sum and the smallest gap
// between neighbours (none for one amount).
struct Run(lo: Vol, hi: Vol, sum: Int, gap: ?Int)

fn join(a: ?Run, b: ?Run) -> ?Run:
    let x = a ?? return b
    let y = b ?? return a
    Some(Run(x.lo, y.hi, x.sum + y.sum, min(x.gap, y.gap, Some(y.lo - x.hi))))

var barrels = AugSortedBag<Vol, Int>((x, y) => x + y, 0, v => Int(v))
var unique = AugSortedSet<Vol, ?Run>(join, None, v => Some(Run(v, v, Int(v), None)))

fn put(v: Vol, op: '+' | '-'):
    if op == '+': barrels.add(v) else: barrels.remove(v)
    if barrels.count(v) == 1: unique.add(v) else: unique.remove(v)

// Before checking the unique amounts in `rest`, every other barrel is safe water.
fn have(rest: Run) -> Int = barrels.all - rest.sum

fn identifiable() -> Bool:
    let last = unique.last ?? return true
    let stuck_points = iterate(from: unique.all, step: rest =>
        let past = Vol.at(max(have(rest), Int(rest.lo)) + 1) ?? return None
        unique.fold_ge(past)
    )
    not stuck_points.any(rest => rest.lo != last and min(Int(rest.lo), rest.gap!) > have(rest))

let (n, q): (1..=200_000, 1..=200_000) = read()
for v in read<[n]Vol>(): put(v, '+')
with io(answers: ("Yes", "No")):
    print(yes(identifiable()))
    repeat q:
        let (op, v): ('+' | '-', Vol) = read()
        put(v, op)
        print(yes(identifiable()))
