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

let MAX_V = 1_000_000
let INF = MAX_V + 1
domain Vol = 0..=MAX_V

var count: [Vol]Int = 0
var total = 0
var total_unique = 0
var unique = SortedSet<Int>()
var unique_sum = Fenwick<Vol, Int>()
var gaps = SegTree<Vol, Int>(Monoid<Int>(INF, (a, b) => min(a, b)))

fn put(v: Vol, delta: Int):
    let was_alone = count[v] == 1
    count[v] += delta
    total += delta * v.ord
    let is_alone = count[v] == 1
    if is_alone == was_alone: return
    if is_alone:
        unique.add(v.ord)
        unique_sum.add(v, v.ord)
        total_unique += v.ord
        let nxt = unique.range(v.ord + 1..).first
        gaps.set(v, if let w = nxt: w - v.ord else: INF)
        if let u = unique.range(..v.ord).last: gaps.set(Vol(u), v.ord - u)
    else:
        unique.remove(v.ord)
        unique_sum.add(v, -v.ord)
        total_unique -= v.ord
        gaps.set(v, INF)
        if let u = unique.range(..v.ord).last:
            let nxt = unique.range(u + 1..).first
            gaps.set(Vol(u), if let w = nxt: w - u else: INF)

fn safe_before(b: Int) -> Int = (total - total_unique) + unique_sum.sum(..Vol(b))

fn identifiable() -> Bool:
    let last_u = unique.last ?? return true
    var cur = unique.first
    while let b = cur:
        if b == last_u: break
        let have = safe_before(b)
        if b > have and gaps.fold(Vol(b)..=Vol.last) > have: return false
        cur = unique.range(max(have, b) + 1..).first
    true

let (n, q): (1..=200_000, 1..=200_000) = read()
repeat n:
    let v: 1..=1_000_000 = read()
    put(Vol(v), 1)
print(if identifiable(): "Yes" else: "No")
repeat q:
    let op: Char = read()
    let v: 1..=1_000_000 = read()
    put(Vol(v), if op == '+': 1 else: -1)
    print(if identifiable(): "Yes" else: "No")
