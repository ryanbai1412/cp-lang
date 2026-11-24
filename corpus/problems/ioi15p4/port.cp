// Source: IOI 2015 Day 2, Horses, https://dmoj.ca/problem/ioi15p4
// Problem: n <= 5e5 years; in year i the herd multiplies by x[i], then horses sell for y[i]
//   each (1 <= x, y <= 1e9). Starting with one horse, print the most money obtainable, mod
//   1e9+7, initially and after each of m <= 1e5 updates setting one x[i] or y[i].
// Solution: it is best to sell everything in one year i, earning x[0] * ... * x[i] * y[i].
//   Walk the years backwards in blocks that start at a year with x > 1 (`surge.left_while`),
//   keeping the best sale so far as a multiple `lead` of the herd at the block start; each
//   block multiplies it by x >= 2, so once it reaches 1e9 no earlier year can win (~30
//   blocks). SegTrees give the max y per block and the prefix product mod p: O(30 log n) each.
// original: corpus/submissions/5723351.cpp (the three grader functions)
// stdin here: n, x, y, m, then m updates `kind year value`, kind 1 setting x and 2 setting y.
type Z = Mod<1_000_000_007>

let n: 1..=500_000 = read()
domain Year = 0..n
var x: [Year]1..=1_000_000_000 = read()
let y: [Year]1..=1_000_000_000 = read()
var growth = SegTree<Year, Z>.of(*, i => Z(x[i]))
var surge = SegTree<Year, Int>(max, [Year] i => Int(x[i]))  // to find the years with x > 1
var price = SegTree<Year, Int>(max, [Year] i => Int(y[i]))

fn best() -> Z:
    var (lead, ans) = (0, Z(0))  // best sale / herd at `end`, and the sale itself
    var end = Year.cuts.last!
    while lead < 1_000_000_000 and end != Year.cuts.first!:
        let quiet = surge.left_while(end, s => s <= 1)
        let start = quiet.left ?? Year.first!
        let top = price.fold(start.l..end)
        if top > lead:
            lead = top
            ans = growth.fold(Year.cuts.first!..start.r) * Z(top)
        lead *= x[start]
        end = start.l
    ans

print(best())
let m: 1..=100_000 = read()
for (kind, at, value) in read<[m](kind: 1..=2, at: Year, value: 1..=1_000_000_000)>():
    if kind == 1:
        x[at] = value
        growth.refresh(at)
        surge.set(at, value)
    else:
        price.set(at, value)
    print(best())
