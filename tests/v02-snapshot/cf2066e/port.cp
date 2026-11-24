// Source: Codeforces 2066E (Tropical Season), https://codeforces.com/contest/2066/problem/E
// Problem: n barrels, a_i <= 1e6, one secretly poisoned. Scales compare any two barrels, and water
//   may be poured from a barrel known to be safe. Can the poisoned barrel always be identified?
//   Answer Yes/No before and after each of q queries "+ x" / "- x" (add/remove a barrel);
//   n, q <= 2e5.
// Solution: barrels whose amount repeats are safe. Check the unique amounts lightest first: with
//   `have` kilograms of safe water, b is checkable if b <= have, or two unchecked barrels if their
//   gap is <= have. It can only get stuck at a unique b > have, and then `have` at least doubles,
//   so O(log sum) stuck points; at each, the unique amounts >= b fold to their sum (so `have`) and
//   their smallest gap. A bag of barrels and a set of unique amounts folded into runs give
//   O(log n log sum) per answer.

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
